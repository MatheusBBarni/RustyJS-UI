import { App, View, Text, Alert } from 'RustyJS-UI';

function calcHeight() {
    Alert({
        title: 'Delete task?',
        description: 'This action cannot be undone.',
        primaryButtonText: 'Delete',
        primaryButtonOnClick: () => {},
        secondaryButtonText: 'Cancel',
        secondaryButtonOnClick: () => {}
    });
}

calcHeight();

function PackageImportScreen() {
    return View({
        style: {
            direction: 'column',
            gap: 12,
            padding: 20
        },
        children: [
            Text({ text: 'Package import fixture' })
        ]
    });
}

App.run({
    title: 'Package Import Fixture',
    windowSize: { width: 480, height: 320 },
    render: PackageImportScreen
});
