import './style.css';
import { App } from './ui/app';

const app = new App(document.getElementById('app')!);
app.init();

if ('serviceWorker' in navigator && location.protocol === 'https:') {
  navigator.serviceWorker.register('./sw.js').catch(() => {});
}
