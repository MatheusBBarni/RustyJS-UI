const VARIANTS = {
    primary: {
        backgroundColor: '#1E5AA8',
        color: '#FFFFFF',
        borderColor: '#1E5AA8'
    },
    secondary: {
        backgroundColor: '#E6EDF6',
        color: '#17324D',
        borderColor: '#C8D5E4'
    },
    danger: {
        backgroundColor: '#B13A3A',
        color: '#FFFFFF',
        borderColor: '#B13A3A'
    },
    ghost: {
        backgroundColor: '#FFFFFF',
        color: '#17324D',
        borderColor: '#D7E0EA'
    }
};

const SIZES = {
    sm: {
        padding: { x: 12, y: 8 },
        fontSize: 14,
        borderRadius: 10
    },
    md: {
        padding: { x: 16, y: 12 },
        fontSize: 16,
        borderRadius: 12
    },
    lg: {
        padding: { x: 18, y: 14 },
        fontSize: 17,
        borderRadius: 14
    }
};

export function AppButton(props = {}) {
    const variant = VARIANTS[props.variant] || VARIANTS.primary;
    const size = SIZES[props.size] || SIZES.md;
    const disabled = Boolean(props.disabled);

    return Button({
        text: props.text || 'Action',
        onClick: disabled ? undefined : props.onClick,
        disabled,
        style: {
            width: props.width,
            height: props.height,
            padding: size.padding,
            fontSize: size.fontSize,
            borderRadius: size.borderRadius,
            borderWidth: 1,
            ...variant,
            ...(props.style || {})
        }
    });
}

