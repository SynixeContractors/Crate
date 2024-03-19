module.exports = {
  content: ["./**/*.{html,js}", "./*.html"],
  theme: {
    extend: {
      colors: {
        synixe: {
          50: "#ffd731",
        }
      },
    },
  },
  plugins: [
    require("@tailwindcss/forms"),
  ],
};
