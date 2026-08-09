/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{vue,js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        // Ancient Chinese ink-wash (古风水墨) theme tokens - 明亮版
        ink: {
          DEFAULT: '#2A1F16',   // 暖褐 background
          deep: '#3A2C20',      // 暖木 surface
          wood: '#4A3A2C',      // 木纹 surface 2
          grain: '#5A4838',     // 木纹 surface 3
          border: '#6E543C',    // 边框
          light: '#8A6E54',
        },
        parchment: {
          DEFAULT: '#EDE0C8',  // 宣纸 text primary
          tan: '#C9B393',       // 旧纸 text secondary
          muted: '#9C8260',     // 淡晕 text muted
          dim: '#6E5A42',
        },
        vermilion: {
          DEFAULT: '#D64531',  // 朱砂 accent（更亮）
          light: '#E86A58',
          dark: '#B5331F',
        },
        gold: {
          DEFAULT: '#E0BA3A',  // 描金（更亮）
          light: '#F0D060',
        },
        jade: {
          DEFAULT: '#5A8E62',  // 青玉 success（更亮）
          light: '#78A87E',
        },
        element: {
          hover: '#5A4838',
          selected: '#4A3A2C',
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