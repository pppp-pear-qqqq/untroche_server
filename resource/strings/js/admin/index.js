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

/**
 * @param {HTMLElement} form 
 */
function update_character(form) {
	const eno = form.querySelector('.eno').innerText;
	const location = form.querySelector('[name="location"]').value;
	ajax.open({
		url: 'admin/update_character',
		ret: 'text',
		post: {eno: Number(eno), location: location},
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
	let id = form.querySelector('.id').innerText;
	let hp = form.querySelector('[name="hp"]').value;
	let mp = form.querySelector('[name="mp"]').value;
	let atk = form.querySelector('[name="atk"]').value;
	let tec = form.querySelector('[name="tec"]').value;
	let skill = form.querySelector('[name="skill"]').value;
	if (make !== true) {
		id = Number(id);
	} else {
		id = null;
	}
	hp = isNaN(hp) ? null : Number(hp);
	mp = isNaN(mp) ? null : Number(mp);
	atk = isNaN(atk) ? null : Number(atk);
	tec = isNaN(tec) ? null : Number(tec);
	skill = isNaN(skill) ? null : Number(skill);
	ajax.open({
		url: 'admin/update_fragment',
		ret: 'text',
		post: {id: id, category: category, name: name, lore: lore, status: {hp: hp, mp: mp, atk: atk, tec: tec}, skill: skill},
		ok: ret => {
			alertify.success(ret);
			form.classList.remove('changed');
			if (make === true) {
				const frame = document.getElementById('template').content.querySelector('div.skill').cloneNode(true);
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