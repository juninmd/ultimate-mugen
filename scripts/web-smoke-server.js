const fs = require('node:fs');
const http = require('node:http');
const path = require('node:path');

function assetFile(root, requestUrl) {
  let requestPath;
  try {
    requestPath = decodeURIComponent(requestUrl.split('?')[0]);
  } catch (error) {
    return null;
  }
  if (requestPath.includes('\0')) return null;
  requestPath = requestPath.replace(/^[\\/]+/, '');
  requestPath = /[\\/]$/.test(requestPath) ? path.join(requestPath, 'index.html') : requestPath;
  const base = path.resolve(root);
  const candidate = path.resolve(base, requestPath || 'index.html');
  return candidate === base || candidate.startsWith(`${base}${path.sep}`) ? candidate : null;
}

function staticServer(root, types) {
  return http.createServer((request, response) => {
    const file = assetFile(root, request.url || '/');
    if (!file) {
      response.writeHead(404).end();
      return;
    }
    fs.readFile(file, (error, data) => {
      if (error) response.writeHead(404).end();
      else response.writeHead(200, { 'content-type': types[path.extname(file)] || 'application/octet-stream' }).end(data);
    });
  });
}

module.exports = { staticServer };
