var hold_element = null;

function get_skill_name(id) {
	if (id == 0) {
		return '';
	} else {
		const skill = document.querySelector(`div.skill[data-id="${id}"]`);
		if (skill !== null) {
			return skill.querySelector('.name').value;
		} else {
			return '取得失敗';
		}
	}
}
function add_changed(event) {
	event.currentTarget.parentNode.classList.add('changed');
}

function execute_sql(text) {
	ajax.open({
		url: 'admin/execute_sql',
		ret: 'text',
		post: {sql: text},
		ok: ret => {
			alertify.success(ret);
		}
	});
	
}
/**
 * 
 * @param {string} text 
 */
function make_fragments_skills(text) {
	text.split(/\s*(\r|\n|\r\n)\s*/).forEach(line => {
		const arg = line.split(/\s*,\s*/);
		switch (arg[0]) {
			case 'f': {
				const status = arg[4].split(/\s+/);
				const params = {
					category: arg[1],
					name: arg[2],
					lore: arg[3],
					hp: Number(status[0]),
					mp: Number(status[1]),
					atk: Number(status[2]),
					tec: Number(status[3]),
					skill: (isNaN(arg[5])) ? null : Number(arg[5]),
				};
				console.log(status, params);
				ajax.open({
					url: 'admin/update_fragment',
					ret: 'text',
					post: params,
					ok: ret => {
						alertify.success(ret);
					}
				});
			} break;
			case 's': {
				let timing = -1;
				switch (arg[3]) {
					case '通常': timing = 0; break;
					case '反応': timing = 1; break;
					case '開始': timing = 2; break;
					case '勝利': timing = 3; break;
					case '敗北': timing = 4; break;
					case '無感': timing = 5; break;
				}
				const params = {
					name: arg[1],
					lore: arg[2],
					timing: timing,
					effect: arg[4],
				};
				ajax.open({
					url: 'admin/update_skill',
					ret: 'text',
					post: params,
					ok: ret => {
						alertify.success(ret);
					}
				});
			} break;
		}
	})
}

function load_characters(container) {
	ajax.open({
		url: 'admin/get_characters',
		ret: 'json',
		ok: ret => {
			const template = document.getElementById('template').content.querySelector('div.character');
			container.replaceChildren();
			ret.forEach(f => {
				const frame = template.cloneNode(true);
				frame.querySelector('.eno').innerText = f['eno'];
				frame.querySelector('.name').innerText = f['name'];
				frame.querySelector('.location').value = f['location'];
				frame.querySelector('.kins').value = f['kins'];
				frame.querySelectorAll('[name]').forEach(elem => elem.addEventListener('change', add_changed));
				container.appendChild(frame);
			});
		}
	})
}
/**
 * @param {HTMLElement} form 
 */
function update_character(form) {
	const eno = Number(form.querySelector('.eno').innerText);
	const location = form.querySelector('[name="location"]').value;
	const kins = form.querySelector('[name="kins"]').value;
	ajax.open({
		url: 'admin/update_character',
		ret: 'text',
		post: {eno: eno, location: location, kins: kins !== '' ? Number(kins) : 0},
		ok: ret => {
			alertify.success(ret);
		}
	});
}

function load_fragments(container) {
	ajax.open({
		url: 'admin/get_fragments',
		ret: 'json',
		ok: ret => {
			const template = document.getElementById('template').content.querySelector('div.fragment');
			container.replaceChildren();
			ret.forEach(f => {
				const frame = template.cloneNode(true);
				frame.querySelector('.id').innerText = f['id'];
				frame.querySelector('.category').value = f['category'];
				frame.querySelector('.name').value = f['name'];
				frame.querySelector('.lore').value = f['lore'];
				frame.querySelector('.hp').value = f['status']['hp'];
				frame.querySelector('.mp').value = f['status']['mp'];
				frame.querySelector('.atk').value = f['status']['atk'];
				frame.querySelector('.tec').value = f['status']['tec'];
				if (f['skill'] !== null) {
					frame.querySelector('.skill').value = f['skill'];
					frame.querySelector('.skill_name').innerText = get_skill_name(f['skill']);
				}
				frame.querySelectorAll('[name]').forEach(elem => elem.addEventListener('change', add_changed));
				container.appendChild(frame);
			});
			const frame = template.cloneNode(true);
			frame.querySelector('.id').innerText = '新規';
			const button = frame.querySelector('button');
			button.innerText = '作成'
			button.onclick = () => update_fragment(frame, true);
			frame.querySelectorAll('[name]').forEach(elem => elem.addEventListener('change', add_changed));
			container.appendChild(frame);
		}
	})
}
/**
 * @param {HTMLElement} form 
 */
function update_fragment(form, make) {
	const category = form.querySelector('[name="category"]').value;
	const name = form.querySelector('[name="name"]').value;
	const lore = form.querySelector('[name="lore"]').value;
	let id = null;
	if (make !== true) id = Number(form.querySelector('.id').innerText);
	const hp = Number(form.querySelector('[name="hp"]').value);
	const mp = Number(form.querySelector('[name="mp"]').value);
	const atk = Number(form.querySelector('[name="atk"]').value);
	const tec = Number(form.querySelector('[name="tec"]').value);
	let skill = Number(form.querySelector('[name="skill"]').value);
	if (skill === 0) skill = null;
	ajax.open({
		url: 'admin/update_fragment',
		ret: 'text',
		post: {id: id, category: category, name: name, lore: lore, status: {hp: hp, mp: mp, atk: atk, tec: tec}, skill: skill},
		ok: ret => {
			alertify.success(ret);
			form.classList.remove('changed');
			if (make === true) {
				const frame = document.getElementById('template').content.querySelector('div.fragment').cloneNode(true);
				frame.querySelector('.id').innerText = frame.dataset.id = Number(form.previousElementSibling.querySelector('.id').innerText) + 1;
				frame.querySelector('.category').value = form.querySelector('.category').value;
				frame.querySelector('.name').value = form.querySelector('.name').value;
				frame.querySelector('.lore').value = form.querySelector('.lore').value;
				frame.querySelector('.hp').value = form.querySelector('.hp').value;
				frame.querySelector('.mp').value = form.querySelector('.mp').value;
				frame.querySelector('.atk').value = form.querySelector('.atk').value;
				frame.querySelector('.tec').value = form.querySelector('.tec').value;
				frame.querySelector('.skill').value = form.querySelector('.skill').value;
				frame.querySelector('.skill_name').value = form.querySelector('.skill_name').value;
				frame.querySelectorAll('[name]').forEach(elem => elem.addEventListener('change', add_changed));
				form.parentNode.insertBefore(frame, form);
				form.querySelectorAll('[name]').forEach(elem => elem.value = '');
			}
		}
	});
}

function load_skills(container) {
	ajax.open({
		url: 'admin/get_skills',
		ret: 'json',
		ok: ret => {
			const template = document.getElementById('template').content.querySelector('div.skill');
			container.replaceChildren();
			ret.forEach(f => {
				const frame = template.cloneNode(true);
				frame.querySelector('.id').innerText = frame.dataset.id = f['id'];
				frame.querySelector('.name').value = f['name'];
				frame.querySelector('.lore').value = f['lore'];
				frame.querySelector('.timing').value = f['timing'];
				frame.querySelector('.effect').value = f['effect'].join(' ');
				frame.querySelectorAll('[name]').forEach(elem => elem.addEventListener('change', add_changed));
				container.appendChild(frame);
			});
			const frame = template.cloneNode(true);
			frame.querySelector('.id').innerText = '新規';
			const button = frame.querySelector('button');
			button.innerText = '作成'
			button.onclick = () => update_skill(frame, true);
			frame.querySelectorAll('[name]').forEach(elem => elem.addEventListener('change', add_changed));
			container.appendChild(frame);
		}
	})
}
/**
 * @param {HTMLElement} form 
 */
function update_skill(form, make) {
	let values = form.querySelectorAll('[name]');
	let params = {};
	values.forEach(v => {
		params[v.getAttribute('name')] = v.value;
	});
	if (make !== true) {
		params['id'] = Number(form.querySelector('.id').innerText);
	}
	params['timing'] = Number(params['timing']);
	console.log(params);
	alertify.message(params['lore']);
	ajax.open({
		url: 'admin/update_skill',
		ret: 'text',
		post: params,
		ok: ret => {
			alertify.success(ret);
			form.classList.remove('changed');
			if (make === true) {
				const frame = document.getElementById('template').content.querySelector('div.skill').cloneNode(true);
				frame.querySelector('.id').innerText = frame.dataset.id = Number(form.previousElementSibling.querySelector('.id').innerText) + 1;
				frame.querySelector('.name').value = form.querySelector('.name').value;
				frame.querySelector('.lore').value = form.querySelector('.lore').value;
				frame.querySelector('.timing').value = form.querySelector('.timing').value;
				frame.querySelector('.effect').value = form.querySelector('.effect').value;
				frame.querySelectorAll('[name]').forEach(elem => elem.addEventListener('change', add_changed));
				form.parentNode.insertBefore(frame, form);
				form.querySelectorAll('[name]').forEach(elem => elem.value = '');
			}
		}
	});
}

function call_load_players_fragments(form) {
	const min = Number(form.querySelector('[name="min"]').value);
	const max = Number(form.querySelector('[name="max"]').value);
	load_players_fragments(form.parentNode, min, max);
}
function load_players_fragments(container, start, end) {
	ajax.open({
		url: 'admin/get_players_fragments',
		ret: 'json',
		get: {start: start, end: end},
		ok: ret => {
			const template = document.getElementById('template').content.querySelector('div.player_fragment');
			const load = document.getElementById('template').content.querySelector('div.load_players_fragments');
			const num = end - start;
			const prev = load.cloneNode(true);
			prev.querySelector('[name="min"]').value = start - num;
			prev.querySelector('[name="max"]').value = start;
			container.replaceChildren(prev);
			ret.forEach(f => {
				const frame = template.cloneNode(true);
				frame.querySelector('.eno').innerText = f['eno'];
				frame.querySelector('.slot').innerText = f['slot'];
				frame.querySelector('.category').value = f['category'];
				frame.querySelector('.user').checked = f['user'];
				frame.querySelector('.name').innerText = f['name'];
				frame.querySelector('.lore').innerHTML = f['lore'];
				frame.querySelector('.hp').value = f['status']['hp'];
				frame.querySelector('.mp').value = f['status']['mp'];
				frame.querySelector('.atk').value = f['status']['atk'];
				frame.querySelector('.tec').value = f['status']['tec'];
				const skill = frame.querySelector('.skill');
				const skill_name = frame.querySelector('.skill_name');
				if (f['skill'] !== null) {
					skill.value = f['skill'];
					skill_name.innerText = get_skill_name(f['skill']);
				}
				skill.addEventListener('change', event => skill_name.innerText = get_skill_name(event.currentTarget.value));
				frame.querySelectorAll('[name]').forEach(elem => elem.addEventListener('change', add_changed));
				container.appendChild(frame);
			});
			const next = load.cloneNode(true);
			next.querySelector('[name="min"]').value = end;
			next.querySelector('[name="max"]').value = end + num;
			container.appendChild(next);
		}
	})
}
function update_player_fragment(form, remove) {
	const eno = Number(form.querySelector('.eno').innerText);
	const slot = Number(form.querySelector('.slot').innerText);
	const category = form.querySelector('[name="category"]').value;
	const user = form.querySelector('[name="user"]').checked;
	let hp = form.querySelector('[name="hp"]').value;
	let mp = form.querySelector('[name="mp"]').value;
	let atk = form.querySelector('[name="atk"]').value;
	let tec = form.querySelector('[name="tec"]').value;
	let skill = form.querySelector('[name="skill"]').value;
	hp = isNaN(hp) ? null : Number(hp);
	mp = isNaN(mp) ? null : Number(mp);
	atk = isNaN(atk) ? null : Number(atk);
	tec = isNaN(tec) ? null : Number(tec);
	skill = isNaN(skill) ? null : Number(skill);
	if (skill === 0) skill = null;
	ajax.open({
		url: 'admin/update_players_fragment',
		ret: 'text',
		post: {delete: remove === true ? remove : false, eno: eno, slot: slot, category: category, status: {hp: hp, mp: mp, atk: atk, tec: tec}, skill: skill, user: user},
		ok: ret => {
			alertify.success(ret);
			if (remove === true) form.remove();
			else form.classList.remove('changed');
		}
	})
}

function add_npc_skill(root) {
	const frame = document.getElementById('template').content.querySelector('div.npc_skill').cloneNode(true);
	frame.ondragstart = event => hold_element = event.currentTarget;
	frame.ondragend = () => hold_element = null;
	frame.ondragenter = event => {
		if (hold_element !== null) {
			const target = event.currentTarget;
			if (target !== hold_element) {
				const parent = target.parentNode;
				const next = target.nextElementSibling;
				if (next === hold_element) {
					parent.insertBefore(hold_element, target);
				} else {
					parent.insertBefore(target, hold_element);
					parent.insertBefore(hold_element, next);
				}
			}
		}
	};
	root.before(frame);
}
function add_reward(root) {
	const frame = document.getElementById('template').content.querySelector('div.reward').cloneNode(true);
	root.before(frame);
}
function load_npcs(container) {
	ajax.open({
		url: 'admin/get_npcs',
		ret: 'json',
		ok: ret => {
			const template = document.getElementById('template');
			const npc = template.content.querySelector('div.npc');
			const skill = template.content.querySelector('div.npc_skill');
			const reward = template.content.querySelector('div.reward');
			container.replaceChildren();
			console.log(ret);
			ret.forEach(f => {
				const frame = npc.cloneNode(true);
				frame.querySelector('.id').innerText = f['id'];
				frame.querySelector('.name').value = f['name'];
				frame.querySelector('.acronym').value = f['acronym'];
				frame.querySelector('.color').value = `#${array_to_colorcode(f['color'])}`;
				frame.querySelector('.start').value = f['word']['start'];
				frame.querySelector('.win').value = f['word']['win'];
				frame.querySelector('.lose').value = f['word']['lose'];
				frame.querySelector('.draw').value = f['word']['draw'];
				frame.querySelector('.escape').value = f['word']['escape'];
				frame.querySelector('.hp').value = f['status']['hp'];
				frame.querySelector('.mp').value = f['status']['mp'];
				frame.querySelector('.atk').value = f['status']['atk'];
				frame.querySelector('.tec').value = f['status']['tec'];
				const skills = frame.querySelector('.skills');
				f['skill'].forEach(v => {
					const frame = skill.cloneNode(true);
					frame.querySelector('.skill').value = v[0];
					frame.querySelector('.skill_name').innerText = get_skill_name(v[0]);
					frame.querySelector('.name').value = v[1];
					frame.querySelector('.word').value = v[2];
					frame.ondragstart = event => hold_element = event.currentTarget;
					frame.ondragend = () => hold_element = null;
					frame.ondragenter = event => {
						if (hold_element !== null) {
							const target = event.currentTarget;
							if (target !== hold_element) {
								const parent = target.parentNode;
								const next = target.nextElementSibling;
								if (next === hold_element) {
									parent.insertBefore(hold_element, target);
								} else {
									parent.insertBefore(target, hold_element);
									parent.insertBefore(hold_element, next);
								}
							}
						}
					};
					skills.appendChild(frame);
				});
				skills.appendChild(make_element('<button type="button" onclick="add_npc_skill(this)">追加</button>'));
				const rewards = frame.querySelector('.rewards');
				f['reward'].forEach(v => {
					const frame = reward.cloneNode(true);
					frame.querySelector('.weight').value = v['id'];
					frame.querySelector('.category').value = v['category'];
					frame.querySelector('.name').value = v['name'];
					frame.querySelector('.lore').value = v['lore'];
					frame.querySelector('.hp').value = v['status']['hp'];
					frame.querySelector('.mp').value = v['status']['mp'];
					frame.querySelector('.atk').value = v['status']['atk'];
					frame.querySelector('.tec').value = v['status']['tec'];
					frame.querySelector('.skill').value = v['skill'];
					frame.querySelector('.skill_name').innerText = get_skill_name(v['skill']);
					rewards.appendChild(frame);
				});
				rewards.appendChild(make_element('<button type="button" onclick="add_reward(this)">追加</button>'));
				frame.querySelectorAll('[name]').forEach(elem => elem.addEventListener('change', add_changed));
				container.appendChild(frame);
			});
			const frame = npc.cloneNode(true);
			frame.querySelector('.id').innerText = '新規';
			frame.querySelector('.skills').appendChild(make_element('<button type="button" onclick="add_npc_skill(this)">追加</button>'));
			frame.querySelector('.rewards').appendChild(make_element('<button type="button" onclick="add_reward(this)">追加</button>'));
			frame.querySelectorAll('[name]').forEach(elem => elem.addEventListener('change', add_changed));
			container.appendChild(frame);
		}
	})
}
function update_npc(form) {
	let id = form.querySelector('.id').innerText;
	let make = isNaN(id) || id === '0';
	if (make) id = null;
	else id = Number(id);
	console.log(make);
	const name = form.querySelector('[name="name"]').value;
	const acronym = form.querySelector('[name="acronym"]').value;
	let color = form.querySelector('[name="color"]').value;
	color = [parseInt(color.slice(1, 3), 16), parseInt(color.slice(3, 5), 16), parseInt(color.slice(5, 7), 16)];
	let start = form.querySelector('[name="start"]').value;
	let win = form.querySelector('[name="win"]').value;
	let lose = form.querySelector('[name="lose"]').value;
	let draw = form.querySelector('[name="draw"]').value;
	let escape = form.querySelector('[name="escape"]').value;
	if (start === '') start = null;
	if (win === '') win = null;
	if (lose === '') lose = null;
	if (draw === '') draw = null;
	if (escape === '') escape = null;
	const hp = Number(form.querySelector('[name="hp"]').value);
	const mp = Number(form.querySelector('[name="mp"]').value);
	const atk = Number(form.querySelector('[name="atk"]').value);
	const tec = Number(form.querySelector('[name="tec"]').value);
	let skill = [];
	form.querySelectorAll('.npc_skill').forEach(form => {
		console.log(form);
		let name = form.querySelector('[name="name"]').value;
		let word = form.querySelector('[name="word"]').value;
		if (name === '') name = null;
		if (word === '') word = null;
		skill.push([Number(form.querySelector('[name="skill"]').value), name, word]);
	})
	let reward = [];
	form.querySelectorAll('.reward').forEach(form => {
		console.log(form);
		const weight = Number(form.querySelector('[name="weight"]').value);
		const category = form.querySelector('[name="category"]').value;
		const name = form.querySelector('[name="name"]').value;
		const lore = form.querySelector('[name="lore"]').value;
		const hp = Number(form.querySelector('[name="hp"]').value);
		const mp = Number(form.querySelector('[name="mp"]').value);
		const atk = Number(form.querySelector('[name="atk"]').value);
		const tec = Number(form.querySelector('[name="tec"]').value);
		let skill = Number(form.querySelector('[name="skill"]').value);
		if (skill === 0) skill = null;
		reward.push({id: weight, category: category, name: name, lore: lore, status: {hp: hp, mp: mp, atk: atk, tec: tec}, skill: skill});
	})
	const params = {id: id, name: name, acronym: acronym, color: color, word: {start: start, win: win, lose: lose, draw: draw, escape: escape}, status: {hp: hp, mp: mp, atk: atk, tec: tec}, skill: skill, reward, reward};
	console.log(params);
	ajax.open({
		url: 'admin/update_npc',
		ret: 'text',
		post: params,
		ok: ret => {
			alertify.success(ret);
			form.classList.remove('changed');
			if (make === true) {
				const frame = form.cloneNode(true);
				frame.querySelector('.id').innerText = frame.dataset.id = Number(form.previousElementSibling.querySelector('.id').innerText) + 1;
				form.before(frame);
				form.querySelectorAll('[name]').forEach(elem => elem.value = '');
			}
		}
	})
}