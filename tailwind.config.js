/** @type {import('tailwindcss').Config} */
module.exports = {
    content: [
      './index.html',
      './src/**/*.{vue,js,ts,jsx,tsx}', // Указываем пути для сканирования классов
    ],
    theme: {
      extend: {
        colors: {
          gray: {
            100: '#f3f4f6', // Явно добавляем gray-100, если он отсутствует
          },
        },
      },
    },
    plugins: [],
  };