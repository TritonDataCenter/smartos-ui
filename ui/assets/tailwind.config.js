module.exports = {
  content: [
      "../templates/**/*.{j2,svg}",
  ],
  theme: {
    extend: {
      colors: {},
      aspectRatio: {}
    },
  },
  plugins: [
    require('@tailwindcss/forms')
  ],
}

