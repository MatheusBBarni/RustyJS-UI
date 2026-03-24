import { SaveButton } from './save_button.js';
import { StatusText } from './status_text.js';

function AppLayout() {
    return View({
        style: {
            direction: 'column',
            gap: 12,
            padding: 18
        },
        children: [
            StatusText(),
            SaveButton()
        ]
    });
}

App.run({
    title: 'Repeated Import Fixture',
    render: AppLayout
});
