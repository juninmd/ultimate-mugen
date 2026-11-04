#requires -Version 5.1
<#
.SYNOPSIS
    Baixa arquivos do MEGA (mega.nz) sem cliente externo, com descriptografia AES-CTR.

.DESCRIPTION
    Varios autores brasileiros publicam chars dublados em links do MEGA. Este
    helper usa a API publica (g.api.mega.co.nz) para obter a URL de download e
    descriptografa o fluxo em AES-128-CTR, conforme o esquema do MEGA.
    Usado por scripts/fetch-ptbr-chars.ps1.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Url,
    [Parameter(Mandatory)][string]$Out
)

$ErrorActionPreference = 'Stop'

if (-not ('MegaDownloader' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.IO;
using System.Net;
using System.Security.Cryptography;
using System.Text;
using System.Text.RegularExpressions;

public static class MegaDownloader
{
    static byte[] B64Url(string s)
    {
        s = s.Replace('-', '+').Replace('_', '/');
        switch (s.Length % 4) { case 2: s += "=="; break; case 3: s += "="; break; }
        return Convert.FromBase64String(s);
    }

    public static long Download(string url, string outPath, Action<long, long> progress)
    {
        string fileId = null, keyStr = null;
        int hash = url.IndexOf('#');
        string before = hash >= 0 ? url.Substring(0, hash) : url;
        if (hash >= 0)
        {
            keyStr = url.Substring(hash + 1).Trim('!');
            if (keyStr.Contains("!"))
            {
                string[] parts = keyStr.Split('!');
                fileId = parts[0];
                keyStr = parts[1];
            }
        }
        if (fileId == null)
        {
            string[] segs = before.TrimEnd('/').Split('/');
            fileId = segs[segs.Length - 1];
        }
        if (string.IsNullOrEmpty(keyStr)) throw new Exception("link MEGA sem chave de descriptografia");

        byte[] k = B64Url(keyStr);
        byte[] aesKey = new byte[16];
        for (int i = 0; i < 16; i++) aesKey[i] = (byte)(k[i] ^ k[16 + i]);
        byte[] nonce = new byte[8];
        Array.Copy(k, 16, nonce, 0, 8);

        // 1. Metadados / URL de download
        string api = "https://g.api.mega.co.nz/cs?id=0&n=" + fileId;
        string body = "[{\"a\":\"g\",\"g\":1}]";
        string json;
        using (var wc = new WebClient())
        {
            wc.Headers[HttpRequestHeader.ContentType] = "application/json";
            json = wc.UploadString(api, "POST", body);
        }
        Match mUrl = Regex.Match(json, "\"g\"\\s*:\\s*\"([^\"]+)\"");
        if (!mUrl.Success) throw new Exception("MEGA respondeu sem URL: " + json);
        long size = 0;
        Match mSize = Regex.Match(json, "\"s\"\\s*:\\s*(\\d+)");
        if (mSize.Success) size = long.Parse(mSize.Groups[1].Value);
        string dl = mUrl.Groups[1].Value.Replace("\\/", "/");

        // 2. Download + AES-128-CTR
        var req = (HttpWebRequest)WebRequest.Create(dl);
        req.UserAgent = "Mozilla/5.0";
        req.Timeout = 120000;
        req.ReadWriteTimeout = 120000;
        using (var resp = (HttpWebResponse)req.GetResponse())
        {
            using (var src = resp.GetResponseStream())
            using (var dst = File.Create(outPath))
            using (var aes = Aes.Create())
            {
                aes.Mode = CipherMode.ECB;
                aes.Padding = PaddingMode.None;
                aes.Key = aesKey;
                using (var enc = aes.CreateEncryptor())
                {
                    const int chunk = 1 << 20; // 1 MB, multiplo de 16
                    byte[] buf = new byte[chunk];
                    byte[] ctr = new byte[chunk];
                    byte[] ks = new byte[chunk];
                    ulong counter = 0;
                    long total = 0;
                    int read;
                    while ((read = src.Read(buf, 0, chunk)) > 0)
                    {
                        int blocks = (read + 15) / 16;
                        int need = blocks * 16;
                        for (int b = 0; b < blocks; b++)
                        {
                            ulong c = counter + (ulong)b;
                            Buffer.BlockCopy(nonce, 0, ctr, b * 16, 8);
                            for (int j = 0; j < 8; j++)
                                ctr[b * 16 + 8 + j] = (byte)(c >> (56 - 8 * j));
                        }
                        enc.TransformBlock(ctr, 0, need, ks, 0);
                        for (int i = 0; i < read; i++) buf[i] ^= ks[i];
                        dst.Write(buf, 0, read);
                        counter += (ulong)blocks;
                        total += read;
                        if (progress != null) progress(total, size);
                    }
                    enc.TransformFinalBlock(new byte[0], 0, 0);
                    return total;
                }
            }
        }
    }
}
'@
}

$result = [MegaDownloader]::Download($Url, $Out, $null)
Write-Host ("[mega] {0:N1} MB -> {1}" -f ($result / 1MB), $Out) -ForegroundColor Green
