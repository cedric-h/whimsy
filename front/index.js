import codeEditor from './custom_elements/codeEditor.js';
import ElmGoogleSignIn from 'elm-google-sign-in/index.js';
import _ from './css/Main.css';

Promise
	.all([
		import('./pkg'),
		import('./src/Main.elm'),
		codeEditor.start()
	])
	.then(([rust, { Elm }, codeEditor]) => {

		let googleSignOutComplete = new EventTarget();

		let elm = document.createElement('div');
		document.body.appendChild(elm);
		var app = Elm.Main.init({
			node: elm,
			flags: googleSignOutComplete,
		});

		let toCanvas = new EventTarget();
		let toElm = new EventTarget();

		toElm.addEventListener("error", e => {
			app.ports.pyErrors.send(e.detail);
		});

		/*
		app.ports.googleSignOut.subscribe(clientId => {
			ElmGoogleSignIn.signOut({
				port: app.ports.googleSignOutComplete,
				clientId: clientId,
			})
		});*/

		app.ports.codeChange.subscribe((data) => {
			toCanvas.dispatchEvent(new CustomEvent("code", { detail: data }));
		});

		window.addEventListener('resize', (() => {
			let resize = () => {
				let canvas = document.getElementsByTagName('canvas')[0];
				canvas.height = window.innerHeight;
				canvas.width = Math.ceil(window.innerWidth/2);
				//toCanvas.dispatchEvent(new CustomEvent("heightResize", { detail: window.innerHeight } ));
			}
			setTimeout(resize, 1);
			return resize;
		})());

		rust.main(1366/2, 768, toCanvas, toElm);
	})
	.catch(console.error);
