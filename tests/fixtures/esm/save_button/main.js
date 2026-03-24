import { SaveButton } from './save_button.js';

function AppLayout() {
    return View({
        style: {
            direction: 'column',
            gap: 12,
            padding: 20
        },
        children: [
            Text({ text: 'Module entry' }),
            SaveButton()
        ]
    });
}

App.run({
    title: 'Save Button Fixture',
    windowSize: { width: 480, height: 320 },
    render: AppLayout
});
