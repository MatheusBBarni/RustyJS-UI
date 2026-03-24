export function SaveButton({ text = 'Save', onClick } = {}) {
    return Button({
        text,
        onClick,
        style: {
            padding: { x: 16, y: 12 },
            borderRadius: 12,
            backgroundColor: '#215B9A',
            color: '#FFFFFF'
        }
    });
}
