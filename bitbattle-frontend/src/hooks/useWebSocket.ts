import { useEffect, useRef, useState, useCallback } from 'react';

interface UseWebSocketOptions {
    url: string;
    onMessage?: (data: string) => void;
    onOpen?: () => void;
    onClose?: () => void;
    onError?: (error: Event) => void;
    shouldReconnect?: boolean;
    reconnectInterval?: number;
    maxReconnectAttempts?: number;
}

export const useWebSocket = ({
                                 url,
                                 onMessage,
                                 onOpen,
                                 onClose,
                                 onError,
                                 shouldReconnect = true,
                                 reconnectInterval = 3000,
                                 maxReconnectAttempts = 5,
                             }: UseWebSocketOptions) => {
    const [isConnected, setIsConnected] = useState<boolean>(false);
    const [reconnectAttempts, setReconnectAttempts] = useState<number>(0);
    const wsRef = useRef<WebSocket | null>(null);
    const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

    const connect = useCallback(() => {
        try {
            const ws = new WebSocket(url);

            ws.onopen = () => {
                setIsConnected(true);
                setReconnectAttempts(0);
                onOpen?.();
            };

            ws.onmessage = (event) => {
                onMessage?.(event.data);
            };

            ws.onclose = () => {
                setIsConnected(false);
                onClose?.();

                if (shouldReconnect && reconnectAttempts < maxReconnectAttempts) {
                    reconnectTimeoutRef.current = setTimeout(() => {
                        setReconnectAttempts(prev => prev + 1);
                        connect();
                    }, reconnectInterval);
                }
            };

            ws.onerror = (error) => {
                onError?.(error);
            };

            wsRef.current = ws;
        } catch (error) {
            console.error('WebSocket connection failed:', error);
        }
    }, [url, onMessage, onOpen, onClose, onError, shouldReconnect, reconnectAttempts, maxReconnectAttempts, reconnectInterval]);

    const sendMessage = useCallback((message: string) => {
        if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
            wsRef.current.send(message);
        }
    }, []);

    const disconnect = useCallback(() => {
        if (reconnectTimeoutRef.current) {
            clearTimeout(reconnectTimeoutRef.current);
            reconnectTimeoutRef.current = undefined;
        }
        if (wsRef.current) {
            wsRef.current.close();
            wsRef.current = null;
        }
    }, []);

    useEffect(() => {
        connect();

        return () => {
            disconnect();
        };
    }, [connect, disconnect]);

    return {
        isConnected,
        sendMessage,
        disconnect,
        reconnectAttempts,
    };
};