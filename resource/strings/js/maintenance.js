
window.addEventListener('load', () => {
	const x = 50;
	const y = 30;
	const game = document.getElementById('game');
	for (let i = 0; i < x; ++i) {
		for (let j = 0; j < y; ++j) {
			const cell = document.createElement('p');
			if (Math.random() < 0.2) {
				cell.classList.add('live');
			}
			game.appendChild(cell);
		}
	}
	setInterval(() => {
		const prev = game.cloneNode(true).childNodes;
		for (let i = 0; i < x; ++i) {
			for (let j = 0; j < y; ++j) {
				let live = 0;
				const xmax = Math.min(i + 1, x - 1);
				for (let nx = Math.max(i - 1, 0); nx <= xmax; ++nx) {
					const ymax = Math.min(j + 1, y - 1);
					for (let ny = Math.max(j - 1, 0); ny <= ymax; ++ny) {
						if (prev.item(nx + ny * x).classList.contains('live')) live += 1;
					}
				}
				const now = i + j * x;
				const cell = prev.item(now);
				if (cell.classList.contains('live') && (live < 3 || live > 4))
					game.children.item(now).classList.remove('live');
				else if (live === 3)
					game.children.item(now).classList.add('live');
			}
		}
	}, 1000);
});