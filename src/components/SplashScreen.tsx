import { useEffect, useState } from 'react';

function SplashScreen() {
  const [visible, setVisible] = useState(true);

  useEffect(() => {
    // Hide splash after 2 seconds (adjust as needed)
    const timer = setTimeout(() => setVisible(false), 2000);
    return () => clearTimeout(timer);
  }, []);

  if (!visible) return null;

  return (
    <div className="fixed inset-0 z-50 bg-neutral-950 text-neutral-100 flex flex-col items-center justify-center animate-fade-in">
      <div className="text-5xl font-extrabold text-primary-500 tracking-wider drop-shadow-lg mb-4">
        OBLIVION
      </div>
      <div className="text-neutral-400 text-sm">Securing your connection...</div>
    </div>
  );
}

export default SplashScreen;
