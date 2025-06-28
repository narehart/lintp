export default {
  plugins: ["prettier-plugin-rust"],
  overrides: [
    {
      files: ["*.yml", "*.yaml"],
      options: {
        tabWidth: 2,
        useTabs: false,
      },
    },
  ],
};
