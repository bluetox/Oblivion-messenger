import { useNavigate } from 'react-router-dom';
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export default function AddProfile() {
  const [step, setStep] = useState(1);
  const [profileName, setProfileName] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [mnemonic, setMnemonic] = useState<string[]>([]);
  const navigate = useNavigate();

  const generateMnemonic = async () => {
    const words: [] = await invoke("generate_mnemonic");
    setMnemonic(words);
  };

  useEffect(() => {
    const handleFocus = (e: Event) => {
      const target = e.target as HTMLElement;
      setTimeout(() => {
        target.scrollIntoView({ behavior: 'smooth', block: 'center' });
      }, 100);
    };

    const inputs = document.querySelectorAll('input');
    inputs.forEach(input => input.addEventListener('focus', handleFocus));
    return () => {
      inputs.forEach(input => input.removeEventListener('focus', handleFocus));
    };
  }, [step]);

  return (
    <div className="min-h-screen w-full flex justify-center items-center bg-neutral-950 text-neutral-100 opacity-0 animate-fade-in">
      <div className="relative w-full max-w-md p-6 bg-neutral-900 rounded-xl shadow-xl space-y-6 mx-4">
        
        {/* Exit button */}
        <button
          onClick={() => navigate('/')}
          className="absolute top-4 right-4 text-3xl text-neutral-400 hover:text-white transition"
        >
          ×
        </button>

        {/* Step indicator */}
        <div className="text-sm text-neutral-400">Step {step} of 4</div>

        {step === 1 && (
          <div className="space-y-4">
            <h1 className="text-2xl font-bold">Add Profile</h1>
            <input
              type="text"
              value={profileName}
              onChange={(e) => setProfileName(e.target.value)}
              placeholder="Profile Name"
              className="w-full p-3 rounded-md bg-neutral-800 text-white placeholder-neutral-500 focus:outline-none focus:ring-2 focus:ring-primary-500"
            />
            <button
              onClick={() => setStep(2)}
              disabled={!profileName}
              className={`w-full py-3 rounded-md text-white transition ${
                profileName ? 'bg-primary-500 hover:bg-primary-600' : 'bg-neutral-700 cursor-not-allowed'
              }`}
            >
              Next
            </button>
          </div>
        )}

        {step === 2 && (
          <div className="space-y-4">
            <h1 className="text-2xl font-bold">Choose Password</h1>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Password"
              className="w-full p-3 rounded-md bg-neutral-800 text-white placeholder-neutral-500 focus:outline-none focus:ring-2 focus:ring-primary-500"
            />
            <button
              onClick={() => setStep(3)}
              disabled={!password}
              className={`w-full py-3 rounded-md text-white transition ${
                password ? 'bg-primary-500 hover:bg-primary-600' : 'bg-neutral-700 cursor-not-allowed'
              }`}
            >
              Next
            </button>
          </div>
        )}

        {step === 3 && (
          <div className="space-y-4">
            <h1 className="text-2xl font-bold">Confirm Password</h1>
            <input
              type="password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              placeholder="Re‑enter Password"
              className="w-full p-3 rounded-md bg-neutral-800 text-white placeholder-neutral-500 focus:outline-none focus:ring-2 focus:ring-primary-500"
            />
            <button
              onClick={() => {
                generateMnemonic();
                setStep(4);
              }}
              disabled={confirmPassword !== password || !confirmPassword}
              className={`w-full py-3 rounded-md text-white transition ${
                confirmPassword === password && confirmPassword
                  ? 'bg-primary-500 hover:bg-primary-600'
                  : 'bg-neutral-700 cursor-not-allowed'
              }`}
            >
              Next
            </button>
          </div>
        )}

        {step === 4 && (
          <div className="space-y-4">
            <h1 className="text-2xl font-bold">Your Mnemonic</h1>
            <p className="text-sm text-neutral-400">Write it down & store safely!</p>
            <div className="grid grid-cols-3 gap-2 text-center">
              {mnemonic.map((word, index) => (
                <div
                  key={index}
                  className="py-2 px-3 bg-neutral-800 rounded-md border border-neutral-700 text-sm"
                >
                  {word}
                </div>
              ))}
            </div>
            <button
              onClick={async () => {
                await invoke("create_profil", {name: profileName, password: password, phrase: mnemonic.join(" ")});
                navigate('/')
              }}
              className="w-full py-3 bg-primary-500 hover:bg-primary-600 text-white rounded-md transition"
            >
              Finish
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
