import { nextSharedCount } from './shared_state.js';

const buttonVersion = nextSharedCount();

export function SaveButton() {
    return Button({
        text: `Save ${buttonVersion}`
    });
}
