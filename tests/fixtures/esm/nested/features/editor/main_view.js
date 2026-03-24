import { saveLabel, titleLabel } from '../shared/labels.js';

export function EditorScreen() {
    return View({
        style: {
            direction: 'column',
            gap: 10,
            padding: 18
        },
        children: [
            Text({ text: titleLabel }),
            Button({ text: saveLabel })
        ]
    });
}
