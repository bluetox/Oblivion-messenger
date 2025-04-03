const { invoke } = window.__TAURI__.core;
async function load_profiles() {
    let profiles = await invoke("get_profiles");

    const profilesDiv = document.getElementById('profiles');
    profilesDiv.innerHTML = '';

    for (const profile of profiles) {

        const profileDiv = document.createElement('div');
        profileDiv.className = 'profile';
        profileDiv.onclick = async () => {
            await selectProfile(profile.profile_name);
        }

        const img = document.createElement('img');
        img.src = "https://wallpapers.com/images/high/netflix-profile-pictures-5yup5hd2i60x7ew3.webp";
        img.alt = profile.profile_name;

        const p = document.createElement('p');
        p.textContent = profile.profile_name;

        profileDiv.appendChild(img);
        profileDiv.appendChild(p);
        profilesDiv.appendChild(profileDiv);
    }
}

async function addProfile() {
    const name = prompt("Enter new profile name:");
    if (!name) return;

    try {
        await invoke("create_profil", { name: name });
        
        const profilesDiv = document.getElementById('profiles');

        const profileDiv = document.createElement('div');
        profileDiv.className = 'profile';
        profileDiv.onclick = async () => {
            await selectProfile(name);
        }

        const img = document.createElement('img');
        img.src = "https://wallpapers.com/images/high/netflix-profile-pictures-5yup5hd2i60x7ew3.webp";  // Fixed URL
        img.alt = name;

        const p = document.createElement('p');
        p.textContent = name;

        profileDiv.appendChild(img);
        profileDiv.appendChild(p);
        profilesDiv.appendChild(profileDiv);
    } catch (error) {
        console.error("Failed to create profile:", error);
    }
}

async function selectProfile(name) {
    await invoke("set_profile_name", {name: name})
    window.location.href = 'chat.html'
}

document.getElementById("add-profile-button").addEventListener('click', addProfile);
load_profiles()