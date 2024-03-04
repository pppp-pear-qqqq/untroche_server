var down = false;
var enemys = null;
var shots = null;
var temp_shot = null;
var temp_enemys = null;

window.addEventListener('load', () => {
	const game = document.getElementById('game');
	enemys = document.getElementById('enemy');
	shots = document.getElementById('shot');
	const template = document.getElementById('template');
	temp_shot = template.content.querySelector('.shot');
	temp_enemys = template.content.querySelectorAll('.enemy');
	const player = document.getElementById('player');
	game.onmousedown = () => down = true;
	game.onmouseup = () => down = false;
	game.ontouchstart = () => down = true;
	game.ontouchend = () => down = false;
	game.onmousemove = event => {
		player.style.left = `${event.clientX}px`;
		player.style.top = `${event.clientY}px`;
	}
	setInterval(() => {
		Array.prototype.forEach.call(shots.children, e => {
			e.style.top = e.style.top - 4;
		})
		Array.prototype.forEach.call(enemys.children, e => {
			e.style.top = e.style.top - e.dataset.speed;
		})
		if (down) {
			const shot = temp_shot.cloneNode(true);
			shot.style.left = player.style.left;
			shot.style.top = player.style.top;
			shots.appendChild(shot);
		}
		if (Math.random() < 0.01) {
			enemys.appendChild(temp_enemys[Math.floor(Math.random() * temp_enemys.length)].cloneNode(true));
		}
	}, 16);
});