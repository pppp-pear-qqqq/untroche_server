function sql() {
	ajax.open({
		url: 'admin/execute_sql',
		ret: 'text',
		post: {sql: document.getElementById('input').value},
		ok: ret => {
			alertify.success(ret);
		}
	});
	
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

function replace_escape_character(text) {
	return text.replaceAll('&lt;', '<').replaceAll('&gt;', '>').replaceAll('&quot;','"').replaceAll('&#039;','\'').replaceAll('&amp;','&');
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
					lore: replace_escape_character(arg[3]),
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
					lore: replace_escape_character(arg[2]),
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
function update_skill(form, make) {
	console.log(form);
	let values = form.querySelectorAll('[name]');
	let params = {};
	values.forEach(v => {
		params[v.getAttribute('name')] = v.value;
	});
	if (make !== true) {
		params['id'] = Number(form.dataset.id);
	}
	params['lore'] = replace_escape_character(params['lore']);
	params['timing'] = Number(params['timing']);
	console.log(params);
	alertify.message(params['lore']);
	ajax.open({
		url: 'admin/update_skill',
		ret: 'text',
		post: params,
		ok: ret => {
			alertify.success(ret);
		}
	});
}
/**
 * @param {HTMLElement} form 
 */
function update_fragment(form, make) {
	let values = form.querySelectorAll('[name]');
	let params = {};
	values.forEach(v => {
		params[v.getAttribute('name')] = v.value;
	});
	if (make !== true) {
		params['id'] = Number(form.dataset.id);
	}
	params['lore'] = replace_escape_character(params['lore']);
	params['hp'] = Number(params['hp']);
	params['mp'] = Number(params['mp']);
	params['atk'] = Number(params['atk']);
	params['tec'] = Number(params['tec']);
	console.log(params);
	alertify.message(params['lore']);
	if (isNaN(params['skill']) || params['skill'] === '0')
		params['skill'] = null;
	else params['skill'] = Number(params['skill']);
	ajax.open({
		url: 'admin/update_fragment',
		ret: 'text',
		post: params,
		ok: ret => {
			alertify.success(ret);
		}
	});
}

window.addEventListener('load', async () => {
	document.querySelectorAll('label.skill>input').forEach(e => {
		if (e.value !== '') {
			const parent = e.parentNode;
			const target = document.querySelector(`.skill[data-id="${e.value}"]>[name="name"]`);
			if (target !== null) {
				parent.append(target.value);
			}
		}
	});
	document.querySelectorAll('[name="timing"]').forEach(e => {
		if (e.dataset.timing !== undefined) {
			e.options[Number(e.dataset.timing)].selected = true;
		}
	});
});