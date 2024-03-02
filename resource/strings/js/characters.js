var loading_profile_eno = null;

/**
 * キャラクターリストを更新
 * @param {HTMLElement} container 
 * @param {Number} num 
 * @param {?Number} start 
 * @param {?string} location 
 */
function load_characters(container, num, start, location) {
	ajax.open({
		url: 'get_characters',
		ret: 'json',
		get: {num, start, location},
		ok: ret => {
			const template = document.getElementById('template_character');
			load(container, ret, i => {
				const e = template.content.cloneNode(true).querySelector('.character');
				e.style.borderColor = `#${array_to_colorcode(i['color'])}`;
				e.onclick = () => {
					if (loading_profile_eno !== Number(i['eno'])) {
						document.getElementById('location').classList.add('hide');
						load_profile(i['eno']);
					}
				};
				e.querySelector('.eno').innerText = `Eno.${i['eno']}`;
				e.querySelector('.acronym').innerText = i['acronym'];
				e.querySelector('.name').innerText = i['name'];
				e.querySelector('.word').innerText = i['comment'];
				return e;
			}, make_element('<div class="character"><p class="word">キャラクターが存在しません</p></div>'));
		}
	});
}
function load_profile(eno) {
	ajax.open({
		url: 'get_profile',
		ret: 'json',
		get: {eno: eno},
		ok: ret => {
			const template = document.getElementById('template_fragment');
			const e = document.querySelector('#profile>div');
			e.querySelector('.eno').innerText = `Eno.${ret['eno']}`;
			e.querySelector('.fullname').innerText = (ret['fullname']!=='') ? ret['fullname'] : '────────────────────────';
			load(e.querySelector('.fragments'), ret['fragments'], i => {
				const e = template.content.cloneNode(true);
				e.querySelector('.name').innerText = i['name'];
				e.querySelector('.category').innerText = i['category'];
				e.querySelector('.lore').innerHTML = i['lore'];
				return e;
			});
			e.querySelector('.acronym').innerText = ret['acronym'];
			e.querySelector('.color').value = `#${array_to_colorcode(ret['color'])}`;
			e.querySelector('.comment').value = ret['comment'];
			e.querySelector('.profile>p').innerHTML = ret['profile'];
			e.querySelector('.memo>p').innerHTML = ret['memo'];
			document.getElementById('profile').classList.remove('hide');
			loading_profile_eno = Number(ret['eno']);
		}
	})
}
function close_profile() {
	document.getElementById('profile').classList.add('hide');
	document.getElementById('location').classList.remove('hide');
	loading_profile_eno = null;
}

window.addEventListener('load', () => {
	load_characters(document.querySelector('#location>div'), 1000, null, '*');
});