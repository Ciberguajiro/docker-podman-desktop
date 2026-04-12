import type { ToastMessage, ToastType } from '../types';

function createToastStore() {
  let toasts = $state<ToastMessage[]>([]);
  let counter = 0;

  return {
    get value() { return toasts; },
    add(message: string, type: ToastType = 'Info') {
      const id = ++counter;
      toasts.push({ id, message, toast_type: type });
      setTimeout(() => this.remove(id), 5000);
    },
    remove(id: number) {
      toasts = toasts.filter(t => t.id !== id);
    },
    success(msg: string) { this.add(msg, 'Success'); },
    error(msg: string) { this.add(msg, 'Error'); },
    info(msg: string) { this.add(msg, 'Info'); },
    warning(msg: string) { this.add(msg, 'Warning'); },
  };
}

export const toastStore = createToastStore();
