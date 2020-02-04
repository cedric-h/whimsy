import codeEditor from './codeEditor.js';
import _ from './css/Main.css';

Promise
	.all([
		import('./pkg'),
		import('./src/Main.elm'),
		codeEditor.start()
	])
	.then(([rust, { Elm }, codeEditor]) => {

		let elm = document.createElement('div');
		document.body.appendChild(elm);
		var app = Elm.Main.init({
			node: elm,
		});

		let toCanvas = new EventTarget();
		let toElm = new EventTarget();

		toElm.addEventListener("error", e => {
			app.ports.pyErrors.send(e.detail);
		});

		app.ports.codeChange.subscribe((data) => {
			toCanvas.dispatchEvent(new CustomEvent("code", { detail: data }));
		});

		let resize = () => {
			let canvas = document.getElementsByTagName('canvas')[0];
			canvas.height = window.innerHeight;
			canvas.width = Math.ceil(window.innerWidth/2);
			//toCanvas.dispatchEvent(new CustomEvent("heightResize", { detail: window.innerHeight } ));
		};
		window.addEventListener('resize', resize);

		setTimeout(resize, 1);
		rust.main(1366/2, 768, toCanvas, toElm);
	})
	.catch(console.error);
