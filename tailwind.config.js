/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ["./index.html", "./src/**/*.rs",],
    theme: {
        extend: {
            transitionTimingFunction: {
                'out-circ': 'cubic-bezier(0, 0.55, 0.45, 1)',
            },
        },
    },
    plugins: [
        function ({ addVariant }) {
            addVariant('child', '& > *');
        }
    ],
}