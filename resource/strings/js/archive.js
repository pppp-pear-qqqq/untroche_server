var timeline = null;
var fragments = null;

function params(form) {
	let params = {};
	form.querySelectorAll('[name]').forEach(elem => {
		let value = elem.value;
		if (elem.getAttribute('nullable') !== null && value === '') value = null;
		else switch (elem.getAttribute('type')) {
			case 'number': value = Number(value); break;
		}
		params[elem.getAttribute('name')] = value;
	});
	return params;
}
function load(elem) {
	return new LoadContainer(elem);
}
class LoadContainer {
	constructor(elem) {
		this.container = elem;
		this.container.replaceChildren();
	}
	timeline(params) {
		ajax.open({
			url: 'archive/get_timeline',
			ret: 'json',
			get: params,
			ok: ret => {
				timeline = ret;
				const template = document.querySelector('template').content.querySelector('div.talk');
				ret.forEach(i => {
					const t = template.cloneNode(true);
					t.querySelector('.id').innerText = i.id;
					t.querySelector('.timestamp').innerText = i.timestamp;
					t.querySelector('.eno').innerText = i.from;
					t.querySelector('.location').innerText = i.location;
					t.querySelector('.acronym').innerText = i.acronym;
					t.querySelector('.name').innerText = i.name;
					t.querySelector('.word').innerHTML = i.word;
					t.querySelector('.to').innerText = i.to;
					t.style.borderColor = array_to_colorcode(i.color);
					this.container.appendChild(t);
				})
			}
		})
	}
	fragments(params) {
		ajax.open({
			url: 'archive/get_fragments',
			ret: 'json',
			get: params,
			ok: ret => {
				fragments = ret;
				const template = document.querySelector('template').content.querySelector('div.fragment');
				for (let i = 1; i <= 30; ++i) {
					const t = template.cloneNode(true);
					let f = ret.find(x => x.slot === i);
					if (f !== undefined) {
						t.querySelector('.slot').innerText = f.slot;
						t.querySelector('.category').innerText = f.category;
						t.querySelector('.name').innerText = f.name;
						t.querySelector('.lore').innerHTML = f.lore;
						const status = t.querySelector('.status');
						status.dataset.hp = (f.status.hp >= 0 ? '+' : '') + f.status.hp;
						status.dataset.mp = (f.status.mp >= 0 ? '+' : '') + f.status.mp;
						status.dataset.atk = (f.status.atk >= 0 ? '+' : '') + f.status.atk;
						status.dataset.tec = (f.status.tec >= 0 ? '+' : '') + f.status.tec;
						status.innerText = `H${status.dataset.hp}, M${status.dataset.mp}, A${status.dataset.atk}, T${status.dataset.tec}`;
						const skill = t.querySelector('.skill');
						if (f.skill != null) {
							skill.querySelector('.name').innerText = f.skill.name;
							skill.querySelector('.default_name').innerText = f.skill.default_name;
							skill.querySelector('.word').innerText = f.skill.word;
							skill.querySelector('.lore').innerHTML = f.skill.lore;
							skill.querySelector('.timing').innerText = f.skill.timing;
							if (f.skill.effect.World != null) skill.querySelector('.effect').innerText = f.skill.effect.World;
							else if (f.skill.effect.Formula != null) skill.querySelector('.effect').innerText = f.skill.effect.Formula.join(' ');
						} else {
							skill.classList.add('none');
						}
					} else {
						t.classList.add('none');
					}
					this.container.appendChild(t);
				}
			}
		})
	}
}

var formula_type = (() => {
	const f = localStorage.getItem('fomula_type');
	if (f === null) {
		localStorage.setItem('fomula_type', 0);
		return 0;
	}
	else return Number(f);
})();
var desc_fragment = null;

function fragment_desc(elem) {
	const desc = document.querySelector('#fragment .desc');
	if (elem !== undefined && desc_fragment !== elem) {
		desc.querySelector('.name').innerText = elem.querySelector('.name').innerText;
		desc.querySelector('.category').innerText = elem.querySelector('.category').innerText;
		desc.querySelector('.lore').innerHTML = elem.querySelector('.lore').innerHTML;
		const status = elem.querySelector('.status');
		desc.querySelector('.status').innerText = `HP${status.dataset.hp}, MP${status.dataset.mp}, ATK${status.dataset.atk}, TEC${status.dataset.tec}`;
		const skill = elem.querySelector('.skill');
		const desc_skill = desc.querySelector('.skill');
		if (!skill.classList.contains('none')) {
			desc_skill.querySelector('.name').innerText = skill.querySelector('.name').innerText;
			desc_skill.querySelector('.default_name').innerText = skill.querySelector('.default_name').innerText;
			desc_skill.querySelector('.word').innerText = skill.querySelector('.word').innerText;
			desc_skill.querySelector('.lore').innerHTML = skill.querySelector('.lore').innerHTML;
			const timing = skill.querySelector('.timing').innerText;
			const effect = skill.querySelector('.effect').innerText;
			desc_skill.querySelector('.timing').innerText = timing;
			const desc_effect = desc_skill.querySelector('.effect');
			if (timing === '世界観' || formula_type == 1) desc_effect.innerText = effect;
			else desc_effect.innerText = make_skillfomula(effect.split(' '));
			desc_skill.classList.remove('hide');
		} else {
			desc_skill.classList.add('hide');
		}
		desc_fragment = elem;
		desc.classList.add('on');
	} else {
		desc.classList.remove('on');
		desc_fragment = null;
	}
}

function change_display(mode, item) {
	document.querySelector('main').className = mode;
	Array.prototype.forEach.call(item.parentNode.children, elem => elem.classList.toggle('select', elem === item));
}

function save(target, mode) {
	const now = Math.floor(new Date().getTime() / 1000);
	const a = document.createElement('a');
	switch (target) {
		case 'timeline': if (timeline !== null) {
			switch (mode) {
				case 'json': {
					a.download = `log${now}.json`;
					a.href = URL.createObjectURL(new Blob([JSON.stringify(timeline)], {type: 'text/plain'}));
				} break;
				case 'html': {
					a.download = `log${now}.html`;
					const data = document.querySelector('#timeline .log').innerHTML;
					a.href = URL.createObjectURL(new Blob([`<!DOCTYPE html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>"Strings".log</title><link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin><link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Noto+Sans+JP:wght@400;800&display=swap"><link rel="stylesheet" type="text/css" href="strings.css"></head><body class="back"><header><p class="system">"Strings".log</p></header><main><div id="timeline"><div class="scroll"><div class="log">${data}</div></div></div></main><footer></footer></body></html>`], {type: 'text/plain'}));
				} break;
				default: alertify.error('モード指定が正しくありません'); return;
			}
		} else {
			alertify.error('ログを読み込んでいません');
			return;
		} break;
		case 'fragments': if (fragments !== null) {
			a.download = `fragment${now}.json`;
			a.href = URL.createObjectURL(new Blob([JSON.stringify(fragments)], {type: 'text/plain'}));
		} else {
			alertify.error('フラグメントを読み込んでいません');
			return;
		} break;
		default: alertify.error('対象指定が正しくありません'); return;
	}
	a.click();
}

window.addEventListener('load', () => {
	const help = document.getElementById('help');
	help.addEventListener('click', event => {
		if (event.target === help) help.classList.add('hide');
	})
	document.querySelectorAll('.help').forEach(elem => {
		elem.addEventListener('click', event => {
			help.querySelector('div').innerHTML = event.currentTarget.innerHTML;
			help.classList.remove('hide');
		});
	});
});