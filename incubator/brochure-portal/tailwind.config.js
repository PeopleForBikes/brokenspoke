/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./pages/**/*.{js,ts,jsx,tsx}",
    "./components/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'pfb-deep-navy': '#00263E',
        'pfb-bright-cyan': '#009EDB',
        'pfb-bright-red': '#D71920',
        'pfb-coal': '#4D4D4F',
        'pfb-gray': '#8B8A8D',
      },
    },
  },
  plugins: [],
}
