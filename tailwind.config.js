/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{vue,js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        // FluxDown dark theme tokens
        surface: {
          DEFAULT: '#1C1C1E',
          1: '#2C2C2E',
          2: '#3A3A3C',
          3: '#48484A',
        },
        element: {
          hover: '#424245',
          selected: '#3A3A3C',
        },
        text: {
          primary: '#F5F5F7',
          secondary: '#A1A1A6',
          muted: '#8E8E93',
          disabled: '#47474A',
        },
      },
      fontFamily: {
        sans: ['-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'Helvetica Neue', 'Arial', 'sans-serif'],
      },
      fontSize: {
        '2xs': '11px',
      },
    },
  },
  plugins: [],
};
