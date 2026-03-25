import { AppButton, EmptyState, ScreenShell, Surface } from '../components/index.js';
import { isAuthenticated } from '../state/app-state.js';

export function NotFoundScreen(route) {
    return ScreenShell({
        title: 'Page not found',
        subtitle: `No route matched ${route.path}.`,
        backgroundColor: '#F7F2E8',
        children: [
            Surface({
                backgroundColor: '#FFFFFF',
                borderColor: '#DCCDBE',
                children: [
                    EmptyState({
                        title: '404',
                        description: 'The requested route is not part of the example.'
                    }),
                    AppButton({
                        text: isAuthenticated() ? 'Go to tasks' : 'Return home',
                        onClick: () => route.navigate(isAuthenticated() ? '/tasks' : '/')
                    })
                ]
            })
        ]
    });
}
