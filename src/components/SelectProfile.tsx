import { useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';
import SplashScreen from './SplashScreen';

type Profile = {
  profile_name: string;
  profile_id: string;
};


function SelectProfile() {
  
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const navigate = useNavigate();

  useEffect(() => {
    (async () => {
      await invoke("terminate_any_client");

      const result = await invoke<Profile[]>('get_profiles');
      setProfiles(result);
    })();
  }, []);

  return (
    <>
    
    <div className="min-h-screen w-full flex justify-center items-center bg-neutral-950 text-neutral-100 opacity-0 animate-fade-in">
      <div className="fixed left-4 top-4">
          <svg
            onClick={() => navigate('/settings-general')}
            className="h-10 w-10 text-neutral-500 cursor-pointer"
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="currentColor"
          >
            <path
              fillRule="evenodd"
              d="M11.828 2.25c-.916 0-1.699.663-1.85 1.567l-.091.549a.798.798 0 0 1-.517.608 7.45 7.45 0 0 0-.478.198.798.798 0 0 1-.796-.064l-.453-.324a1.875 1.875 0 0 0-2.416.2l-.243.243a1.875 1.875 0 0 0-.2 2.416l.324.453a.798.798 0 0 1 .064.796 7.448 7.448 0 0 0-.198.478.798.798 0 0 1-.608.517l-.55.092a1.875 1.875 0 0 0-1.566 1.849v.344c0 .916.663 1.699 1.567 1.85l.549.091c.281.047.508.25.608.517.06.162.127.321.198.478a.798.798 0 0 1-.064.796l-.324.453a1.875 1.875 0 0 0 .2 2.416l.243.243c.648.648 1.67.733 2.416.2l.453-.324a.798.798 0 0 1 .796-.064c.157.071.316.137.478.198.267.1.47.327.517.608l.092.55c.15.903.932 1.566 1.849 1.566h.344c.916 0 1.699-.663 1.85-1.567l.091-.549a.798.798 0 0 1 .517-.608 7.52 7.52 0 0 0 .478-.198.798.798 0 0 1 .796.064l.453.324a1.875 1.875 0 0 0 2.416-.2l.243-.243c.648-.648.733-1.67.2-2.416l-.324-.453a.798.798 0 0 1-.064-.796c.071-.157.137-.316.198-.478.1-.267.327-.47.608-.517l.55-.091a1.875 1.875 0 0 0 1.566-1.85v-.344c0-.916-.663-1.699-1.567-1.85l-.549-.091a.798.798 0 0 1-.608-.517 7.507 7.507 0 0 0-.198-.478.798.798 0 0 1 .064-.796l.324-.453a1.875 1.875 0 0 0-.2-2.416l-.243-.243a1.875 1.875 0 0 0-2.416-.2l-.453.324a.798.798 0 0 1-.796.064 7.462 7.462 0 0 0-.478-.198.798.798 0 0 1-.517-.608l-.091-.55a1.875 1.875 0 0 0-1.85-1.566h-.344ZM12 15.75a3.75 3.75 0 1 0 0-7.5 3.75 3.75 0 0 0 0 7.5Z"
              clipRule="evenodd"
            />
          </svg>
      </div>
      <div className="bg-neutral-900 border border-neutral-800 rounded-2xl p-8 w-full max-w-md shadow-xl space-y-6 mx-4">
        <h1 className="text-2xl font-bold text-center">Select a Profile</h1>

        <div className="grid grid-cols-2 gap-4">
          {profiles.map((profile) => (
            <div
              key={profile.profile_id}
              onClick={async () => {
                await invoke("set_profile_name", { name: profile.profile_name });
                navigate('/password')
              }}
              className="flex flex-col items-center justify-center p-4 bg-primary-800 rounded-xl shadow-md hover:shadow-lg hover:-translate-y-1 transition cursor-pointer"
            >
              <div className="w-20 h-20 rounded-full bg-primary-600 border-2 border-primary-400 flex items-center justify-center text-xl text-white font-bold mb-2">
                {profile.profile_name[0]?.toUpperCase() || '?'}
              </div>
              <div className="text-sm font-medium truncate">{profile.profile_name}</div>
            </div>
          ))}
        </div>

        <button
          onClick={() => navigate('/add')}
          className="w-full py-3 bg-primary-500 hover:bg-primary-600 rounded-md font-semibold text-white transition"
        >
          + Add Profile
        </button>
      </div>
    </div>
    </>
  );
}

export default SelectProfile;
