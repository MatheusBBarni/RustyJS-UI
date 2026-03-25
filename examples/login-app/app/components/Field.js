export function FieldLabel(props = {}) {
    return Text({
        text: props.text || '',
        style: {
            fontSize: 14,
            color: props.color || '#425466'
        }
    });
}

export function FieldHint(props = {}) {
    return Text({
        text: props.text || '',
        style: {
            fontSize: 13,
            color: props.color || '#6B7D90'
        }
    });
}

export function FieldError(props = {}) {
    return Text({
        text: props.text || '',
        style: {
            fontSize: 13,
            color: props.color || '#B13A3A'
        }
    });
}

export function TextField(props = {}) {
    return View({
        style: {
            width: props.width || 'fill',
            flexDirection: 'column',
            gap: 6,
            ...(props.style || {})
        },
        children: [
            ...(props.label ? [FieldLabel({ text: props.label })] : []),
            TextInput({
                value: props.value || '',
                placeholder: props.placeholder || '',
                onChange: props.onChange,
                disabled: Boolean(props.disabled),
                multiline: Boolean(props.multiline),
                style: {
                    width: props.inputWidth || 'fill',
                    padding: 12,
                    borderWidth: 1,
                    borderRadius: 12,
                    borderColor: props.error ? '#D99A9A' : '#C7D2DE',
                    backgroundColor: props.backgroundColor || '#FFFFFF',
                    color: props.color || '#102033',
                    ...(props.inputStyle || {})
                }
            }),
            ...(props.hint ? [FieldHint({ text: props.hint })] : []),
            ...(props.error ? [FieldError({ text: props.error })] : [])
        ]
    });
}

export function SelectField(props = {}) {
    return View({
        style: {
            width: props.width || 'fill',
            flexDirection: 'column',
            gap: 6,
            ...(props.style || {})
        },
        children: [
            ...(props.label ? [FieldLabel({ text: props.label })] : []),
            SelectInput({
                value: props.value || '',
                placeholder: props.placeholder || '',
                options: Array.isArray(props.options) ? props.options : [],
                onChange: props.onChange,
                disabled: Boolean(props.disabled),
                style: {
                    width: props.inputWidth || 'fill',
                    padding: 12,
                    borderWidth: 1,
                    borderRadius: 12,
                    borderColor: props.error ? '#D99A9A' : '#C7D2DE',
                    backgroundColor: props.backgroundColor || '#FFFFFF',
                    color: props.color || '#102033',
                    ...(props.inputStyle || {})
                }
            }),
            ...(props.hint ? [FieldHint({ text: props.hint })] : []),
            ...(props.error ? [FieldError({ text: props.error })] : [])
        ]
    });
}
