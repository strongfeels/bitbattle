import '@testing-library/jest-dom';
import { afterEach } from 'vitest';
import { cleanup } from '@testing-library/react';

// Cleanup after each test
afterEach(() => {
    cleanup();
});

// Mock window.matchMedia for tests
Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: (query: string) => ({
        matches: false,
        media: query,
        onchange: null,
        addListener: () => {},
        removeListener: () => {},
        addEventListener: () => {},
        removeEventListener: () => {},
        dispatchEvent: () => false,
    }),
});

// Mock WebSocket for tests
const MockWebSocket = function(this: any, url: string) {
    this.url = url;
    this.readyState = 1; // OPEN
    this.onopen = null;
    this.onclose = null;
    this.onmessage = null;
    this.onerror = null;

    setTimeout(() => {
        if (this.onopen) this.onopen();
    }, 0);
} as any;

MockWebSocket.CONNECTING = 0;
MockWebSocket.OPEN = 1;
MockWebSocket.CLOSING = 2;
MockWebSocket.CLOSED = 3;

MockWebSocket.prototype.send = function(_data: string) {
    // Mock send
};

MockWebSocket.prototype.close = function() {
    this.readyState = MockWebSocket.CLOSED;
    if (this.onclose) this.onclose();
};

Object.defineProperty(window, 'WebSocket', {
    writable: true,
    value: MockWebSocket,
});
