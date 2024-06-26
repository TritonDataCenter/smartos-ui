module.exports = {
  content: [
    '../templates/**/*.{j2,svg}'
  ],
  theme: {
    extend: {
      /*
       * Generated from the blue in the SmartOS logo using:
       * https://colorkit.co/color/24b8eb/
       * https://www.tints.dev/blue/24B8EB
       */
      colors: {
        blue: {
          50: '#F2FAFC',
          100: '#E6F6F9',
          200: '#CDECF4',
          300: '#B4E3EE',
          400: '#9BD9E9',
          500: '#80CFE3',
          600: '#47BAD6',
          700: '#2794B0',
          800: '#1A6375',
          900: '#0D313B',
          950: '#06191D'
        },
        amber: {
          50: '#FDEDE8',
          100: '#FBDED5',
          200: '#F7BAA6',
          300: '#F39A7C',
          400: '#EF764D',
          500: '#EB5724',
          600: '#C43E12',
          700: '#96300E',
          800: '#621F09',
          900: '#331005',
          950: '#170702'
        },
        purple: {
          50: '#F8E8FD',
          100: '#F2D5FB',
          200: '#E3A6F7',
          300: '#D57CF3',
          400: '#C64DEF',
          500: '#BA24EB',
          600: '#9812C4',
          700: '#740E96',
          800: '#4C0962',
          900: '#280533',
          950: '#120217'
        },
        green: {
          50: '#EDFDE8',
          100: '#DEFBD5',
          200: '#B9F7A6',
          300: '#98F37C',
          400: '#73EF4D',
          500: '#54EB24',
          600: '#3CC412',
          700: '#2D960E',
          800: '#1E6209',
          900: '#103305',
          950: '#071702'
        },
        red: {
          50: '#FDE9E8',
          100: '#FBD8D5',
          200: '#F7ADA6',
          300: '#F3867C',
          400: '#EF5B4D',
          500: '#EB3624',
          600: '#C42112',
          700: '#96190E',
          800: '#621009',
          900: '#330905',
          950: '#170402'
        },
        yellow: {
          50: '#FDFBE8',
          100: '#FBF8D5',
          200: '#F7F0A6',
          300: '#F3E97C',
          400: '#EFE14D',
          500: '#EBDC24',
          600: '#C4B512',
          700: '#968A0E',
          800: '#625B09',
          900: '#333005',
          950: '#171602'
        }
      },
      aspectRatio: {}
    }
  },
  plugins: [
    require('@tailwindcss/forms'),
    require('postcss-import')
  ]
}
