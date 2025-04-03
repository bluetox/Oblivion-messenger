const { invoke } = window.__TAURI__.core;

async function addProfile() {
    const name = prompt("Enter new profile name:");
    await invoke("create_profil", {name: name})
}
document.getElementById("test-profile").onclick = async () => {
    await selectProfile("mainprofile");
}
document.getElementById("add-profile-button").addEventListener('click', addProfile);

async function selectProfile(name) {
    await invoke("set_profile_name", {name: name})
    window.location.href = 'chat.html'
}
