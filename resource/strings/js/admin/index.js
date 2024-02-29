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
	params['timing'] = Number(params['timing']);
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
	params['hp'] = Number(params['hp']);
	params['mp'] = Number(params['mp']);
	params['atk'] = Number(params['atk']);
	params['tec'] = Number(params['tec']);
	params['skill'] = Number(params['skill']);
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