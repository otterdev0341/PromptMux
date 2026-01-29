/** @type {import('tailwindcss').Config} */
export default {
  content: {
    files: ['./src/**/*.{html,js,svelte,ts}'],
  },
  theme: {
    extend: {
      colors: {
        gray: {
          750: '#2d3748',
          850: '#1a202c',
          950: '#0d1117',
        },
      },
      fontFamily: {
        mono: ['"Fira Code"', '"Monaco"', '"Consolas"', 'monospace'],
      },
    },
  },
  plugins: [],
  darkMode: 'class',
}
