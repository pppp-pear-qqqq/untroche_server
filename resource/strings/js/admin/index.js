function make_skill() {
	let v = document.getElementById('input').value;
	const output = document.getElementById('output');
	output.innerHTML = '';
	v.split('\n').forEach(line => {
		if(line !== '') ajax.open({
			url: 'admin/untroche/make_skill',
			ret: 'text',
			post: {value: line},
			ok: (ret) => {
				output.innerHTML += ret + '<br>';
			}
		});
	});
}
function make_fragment() {
	let v = document.getElementById('input').value;
	const output = document.getElementById('output');
	output.innerHTML = '';
	console.log(v);
	v.split('\n').forEach(line => {
		if(line !== '') ajax.open({
			url: 'admin/untroche/make_fragment',
			ret: 'text',
			post: {value: line},
			ok: (ret) => {
				output.innerHTML += ret + '<br>';
			}
		});
	})
}