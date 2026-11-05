const fs = require('node:fs');
const http = require('node:http');
const path = require('node:path');

async function assetFile(root, requestUrl) {
  let requestPath;
  try {
    requestPath = decodeURIComponent(requestUrl.split('?')[0]);
  } catch (error) {
    return null;
  }
  if (requestPath.includes('\\0')) return null;
  requestPath = requestPath.replace(/^[\\/]+/, '');
  requestPath = /[\\/]$/.test(requestPath) ? path.join(requestPath, 'index.html') : requestPath;
  const base = path.resolve(root);
  const candidate = path.resolve(base, requestPath || 'index.html');
  if (candidate !== base && !candidate.startsWith(`${base}${path.sep}`)) return null;
  try {
    const [realBase, realCandidate] = await Promise.all([fs.promises.realpath(base), fs.promises.realpath(candidate)]);
    const relative = path.relative(realBase, realCandidate);
    return relative === '..' || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative) ? null : realCandidate;
  } catch (error) {
    return null;
  }
}

function staticServer(root, types) {
  return http.createServer((request, response) => {
    assetFile(root, request.url || '/').then(file => {
      if (!file) {
        response.writeHead(404).end();
        return;
      }
      fs.readFile(file, (error, data) => {
        if (error) response.writeHead(404).end();
        else response.writeHead(200, { 'content-type': types[path.extname(file)] || 'application/octet-stream' }).end(data);
      });
    }).catch(() => response.writeHead(404).end());
  });
}

module.exports = { staticServer };
