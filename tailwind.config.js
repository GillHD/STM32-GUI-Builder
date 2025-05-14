/** @type {import('tailwindcss').Config} */
module.exports = {
    content: [
      './index.html',
      './src/**/*.{vue,js,ts,jsx,tsx}', // Path to your frontend components
    ],
    theme: {
      extend: {
        colors: {
          gray: {
            100: '#f3f4f6', // Light gray setup
          },
        },
      },
    },
    plugins: [],
  };