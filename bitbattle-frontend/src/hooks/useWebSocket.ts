import { useEffect, useRef, useState } from 'react';

interface UseWebSocketOptions {
    url: string;
    onMessage?: (data: string) => void;
    onOpen?: () => void;
    onClose?: () => void;
    onError?: (error: Event) => void;
    shouldReconnect?: boolean;
}

export const useWebSocket = (options: UseWebSocketOptions) => {
    const [isConnected, setIsConnected] = useState(false);
    const wsRef = useRef<WebSocket | null>(null);
    const optionsRef = useRef(options);

    // Update options ref when they change
    useEffect(() => {
        optionsRef.current = options;
    }, [options.onMessage, options.onOpen, options.onClose, options.onError]);

    const sendMessage = (message: string) => {
        if (wsRef.current?.readyState === WebSocket.OPEN) {
            wsRef.current.send(message);
            console.log('📤 Message sent successfully');
        } else {
            console.warn('⚠️ Cannot send message - WebSocket not connected');
        }
    };

    useEffect(() => {
        console.log('🚀 Connecting to WebSocket:', options.url);

        const ws = new WebSocket(options.url);
        wsRef.current = ws;

        ws.onopen = () => {
            console.log('✅ WebSocket connected');
            setIsConnected(true);
            optionsRef.current.onOpen?.();
        };

        ws.onmessage = (event) => {
            console.log('📥 WebSocket message received');
            optionsRef.current.onMessage?.(event.data);
        };

        ws.onclose = (event) => {
            console.log('❌ WebSocket disconnected:', event.code, event.reason);
            setIsConnected(false);
            optionsRef.current.onClose?.();
        };

        ws.onerror = (error) => {
            console.error('❌ WebSocket error:', error);
            optionsRef.current.onError?.(error);
        };

        // Cleanup function
        return () => {
            console.log('🧹 Cleaning up WebSocket');
            if (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING) {
                ws.close(1000, 'Component cleanup');
            }
            wsRef.current = null;
            setIsConnected(false);
        };
    }, [options.url]); // Only reconnect when URL changes

    return {
        isConnected,
        sendMessage,
        disconnect: () => {
            wsRef.current?.close(1000, 'Manual disconnect');
        },
        reconnectAttempts: 0,
    };
};