module.exports = {
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#f9f0ff',
          100: '#f3d9ff',
          200: '#e5b3ff',
          300: '#d68cff',
          400: '#c766ff',
          500: '#b300ff',
          600: '#9900cc',
          700: '#7a0099',
          800: '#5c0066',
          900: '#3d0033',
        },
        accent: {
          100: '#d7f0f9',
          300: '#74c4e1',
          500: '#00aaff',
          700: '#183556',
        },
        neutral: {
          100: '#f8f8fa',
          200: '#e4e4e7',
          500: '#6b7280',
          800: '#1e1e2e',
          900: '#0f0f1a',
        },
      },
      keyframes: {
        'fade-in': {
          '0%': { opacity: 0 },
          '100%': { opacity: 1 },
        },
      },
      animation: {
        'fade-in': 'fade-in 0.2s ease-out forwards',
      },
    }

  }
}