import { useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';

function EnterPassword() {
  const navigate = useNavigate();
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async () => {
    try {
      setLoading(true);
      const userId = await invoke("generate_dilithium_keys", { password });
      if (userId) {
        navigate('/chat-list');
      } else {
        alert("Invalid password or error during authentication.");
      }
    } catch (error) {
      console.error(error);
      alert("An error occurred.");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen w-full flex justify-center items-center bg-neutral-950 text-neutral-100 opacity-0 animate-fade-in relative">
      <div className="bg-neutral-900 border border-neutral-800 rounded-2xl p-8 w-full max-w-sm shadow-xl space-y-6 z-10">
        <h1 className="text-xl font-semibold text-center">Enter Your Password</h1>

        <input
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          placeholder="Password"
          className="w-full px-4 py-3 rounded-lg border border-neutral-700 bg-neutral-800 text-neutral-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
        />

        <button
          onClick={handleSubmit}
          disabled={!password || loading}
          className="w-full py-3 bg-primary-500 hover:bg-primary-600 disabled:opacity-50 disabled:cursor-not-allowed rounded-md font-semibold text-white transition"
        >
          {loading ? "Loading..." : "Enter"}
        </button>
      </div>

      {/* Loading overlay */}
      {loading && (
        <div className="absolute inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50">
          <div className="w-12 h-12 border-4 border-white border-t-transparent rounded-full animate-spin" />
        </div>
      )}
    </div>
  );
}

export default EnterPassword;
