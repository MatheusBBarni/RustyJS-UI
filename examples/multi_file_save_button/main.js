import { SaveButton } from './save_button.js';

let saveCount = 0;

function handleSave() {
    saveCount += 1;
    App.requestRender();
}

function AppLayout() {
    return View({
        style: {
            width: 'fill',
            height: 'fill',
            padding: 24,
            direction: 'column',
            gap: 16,
            backgroundColor: '#F4F7FA'
        },
        children: [
            Text({
                text: 'Multi-file SaveButton',
                style: {
                    fontSize: 28,
                    color: '#102033'
                }
            }),
            Text({
                text: `Saves: ${saveCount}`,
                style: {
                    fontSize: 18,
                    color: '#425466'
                }
            }),
            SaveButton({
                text: 'Save changes',
                onClick: handleSave
            })
        ]
    });
}

App.run({
    title: 'Multi-file SaveButton Example',
    windowSize: { width: 520, height: 320 },
    render: AppLayout
});
