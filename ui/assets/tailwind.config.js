module.exports = {
  content: [
      "../templates/**/*.{j2,svg}",
  ],
  theme: {
    extend: {
      // Generated from the blue in the SmartOS logo using:
      // https://www.tints.dev/blue/24B8EB
      colors: {
        blue: {
          50: "#F2FAFC",
          100: "#E6F6F9",
          200: "#CDECF4",
          300: "#B4E3EE",
          400: "#9BD9E9",
          500: "#80CFE3",
          600: "#47BAD6",
          700: "#2794B0",
          800: "#1A6375",
          900: "#0D313B",
          950: "#06191D"
        }
      },
      aspectRatio: {}
    },
  },
  plugins: [
    require('@tailwindcss/forms')
  ],
}

