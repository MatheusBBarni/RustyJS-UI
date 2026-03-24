import { counter } from './shared_state.js';

export function StatusText() {
    return Text({
        text: `Shared count: ${counter}`
    });
}
