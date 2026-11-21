async function analyzeShot(page, png) {
  const stats = await page.evaluate(async b64 => {
    const blob = await (await fetch(`data:image/png;base64,${b64}`)).blob();
    const bitmap = await createImageBitmap(blob);
    const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
    const ctx = canvas.getContext('2d');
    ctx.drawImage(bitmap, 0, 0);
    const data = ctx.getImageData(0, 0, bitmap.width, bitmap.height).data;
    let sum = 0, sum2 = 0, n = 0;
    for (let i = 0; i < data.length; i += 28) { const l = (data[i] + data[i + 1] + data[i + 2]) / 3; sum += l; sum2 += l * l; n++; }
    const mean = sum / n;
    return { mean, deviation: Math.sqrt(Math.max(0, sum2 / n - mean * mean)) };
  }, png.toString('base64'));
  return { blank: stats.mean < 6 || stats.deviation < 4, describe: `luminance mean ${stats.mean.toFixed(1)}, deviation ${stats.deviation.toFixed(1)}` };
}

function markdown(report) {
  const lines = [`# Smoke report: ${report.passed ? 'PASS' : 'FAIL'}`, '', `Platform: ${report.platform}`, ''];
  if (report.panic) lines.push(`**Problems:** ${report.panic}`, '');
  lines.push('| Feature | Result | Frames | Avg ms | Max ms |', '|---|---|---|---|---|');
  for (const f of report.features) lines.push(`| ${f.title} | ${f.ok ? 'pass' : 'FAIL'} | ${f.frames} | ${f.frame_avg_ms} | ${f.frame_max_ms} |`);
  for (const f of report.features.filter(f => !f.ok)) {
    lines.push('', `## ${f.title} failed`, '');
    for (const c of f.checks.filter(c => !c.ok)) lines.push(`- ${c.name}: ${c.detail}${c.shot ? ` ![](${c.shot})` : ''}`);
  }
  return lines.join('\n') + '\n';
}

module.exports = { analyzeShot, markdown };
