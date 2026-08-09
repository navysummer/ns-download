/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{vue,js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        // Ancient Chinese ink-wash (古风水墨) theme tokens
        ink: {
          DEFAULT: '#1A1109',   // 深墨 background
          deep: '#241A10',      // 深木 surface
          wood: '#33271C',      // 木纹 surface 2
          grain: '#423122',     // 木纹 surface 3
          border: '#5A4330',    // 边框
          light: '#6E543C',
        },
        parchment: {
          DEFAULT: '#EDE0C8',  // 宣纸 text primary
          tan: '#C9B393',       // 旧纸 text secondary
          muted: '#9C8260',     // 淡晕 text muted
          dim: '#6E5A42',
        },
        vermilion: {
          DEFAULT: '#B53A2E',  // 朱砂 accent
          light: '#D05547',
          dark: '#8E2B22',
        },
        gold: {
          DEFAULT: '#C9A227',  // 描金
          light: '#E0C25C',
        },
        jade: {
          DEFAULT: '#4E7A5A',  // 青玉 success
          light: '#6B9A74',
        },
        element: {
          hover: '#423122',
          selected: '#33271C',
        },
        text: {
          primary: '#EDE0C8',
          secondary: '#C9B393',
          muted: '#9C8260',
          disabled: '#6E5A42',
        },
      },
      fontFamily: {
        sans: ['"Kaiti SC"', '"STKaiti"', '"KaiTi"', '"Songti SC"', '"STSong"', '"Noto Sans CJK SC"', '"PingFang SC"', 'serif'],
        kai: ['"Kaiti SC"', '"STKaiti"', '"KaiTi"', 'serif'],
        song: ['"Songti SC"', '"STSong"', '"SimSun"', 'serif'],
        seal: ['"Xingkai SC"', '"STXingkai"', '"Kaiti SC"', 'serif'],
      },
      fontSize: {
        '2xs': '11px',
      },
    },
  },
  plugins: [],
};