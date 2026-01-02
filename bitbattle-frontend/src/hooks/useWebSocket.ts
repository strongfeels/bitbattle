import { useEffect, useRef, useState, useCallback } from 'react';

interface UseWebSocketOptions {
    url: string;
    onMessage?: (data: string) => void;
    onOpen?: () => void;
    onClose?: () => void;
    onError?: (error: Event) => void;
    shouldReconnect?: boolean;
    maxReconnectAttempts?: number;
    reconnectInterval?: number;
}

export const useWebSocket = (options: UseWebSocketOptions) => {
    const {
        url,
        shouldReconnect = true,
        maxReconnectAttempts = 5,
        reconnectInterval = 1000,
    } = options;

    const [isConnected, setIsConnected] = useState(false);
    const [reconnectAttempts, setReconnectAttempts] = useState(0);
    const [connectionState, setConnectionState] = useState<'connecting' | 'connected' | 'disconnected' | 'reconnecting'>('connecting');

    const wsRef = useRef<WebSocket | null>(null);
    const optionsRef = useRef(options);
    const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
    const manualDisconnectRef = useRef(false);

    // Update options ref when they change
    useEffect(() => {
        optionsRef.current = options;
    }, [options.onMessage, options.onOpen, options.onClose, options.onError]);

    const clearReconnectTimeout = useCallback(() => {
        if (reconnectTimeoutRef.current) {
            clearTimeout(reconnectTimeoutRef.current);
            reconnectTimeoutRef.current = null;
        }
    }, []);

    const connect = useCallback(() => {
        if (wsRef.current?.readyState === WebSocket.OPEN || wsRef.current?.readyState === WebSocket.CONNECTING) {
            return;
        }

        console.log('🚀 Connecting to WebSocket:', url);
        setConnectionState('connecting');

        const ws = new WebSocket(url);
        wsRef.current = ws;

        ws.onopen = () => {
            console.log('✅ WebSocket connected');
            setIsConnected(true);
            setReconnectAttempts(0);
            setConnectionState('connected');
            manualDisconnectRef.current = false;
            optionsRef.current.onOpen?.();
        };

        ws.onmessage = (event) => {
            console.log('📥 WebSocket message received');
            optionsRef.current.onMessage?.(event.data);
        };

        ws.onclose = (event) => {
            console.log('❌ WebSocket disconnected:', event.code, event.reason);
            setIsConnected(false);
            wsRef.current = null;
            optionsRef.current.onClose?.();

            // Don't reconnect if manually disconnected or if shouldReconnect is false
            if (manualDisconnectRef.current || !shouldReconnect) {
                setConnectionState('disconnected');
                return;
            }

            // Attempt reconnection with exponential backoff
            setReconnectAttempts((prev) => {
                const nextAttempt = prev + 1;

                if (nextAttempt <= maxReconnectAttempts) {
                    setConnectionState('reconnecting');
                    const delay = reconnectInterval * Math.pow(2, prev); // Exponential backoff
                    console.log(`🔄 Reconnecting in ${delay}ms (attempt ${nextAttempt}/${maxReconnectAttempts})`);

                    clearReconnectTimeout();
                    reconnectTimeoutRef.current = setTimeout(() => {
                        connect();
                    }, delay);
                } else {
                    console.log('❌ Max reconnection attempts reached');
                    setConnectionState('disconnected');
                }

                return nextAttempt;
            });
        };

        ws.onerror = (error) => {
            console.error('❌ WebSocket error:', error);
            optionsRef.current.onError?.(error);
        };
    }, [url, shouldReconnect, maxReconnectAttempts, reconnectInterval, clearReconnectTimeout]);

    const sendMessage = useCallback((message: string) => {
        if (wsRef.current?.readyState === WebSocket.OPEN) {
            wsRef.current.send(message);
            console.log('📤 Message sent successfully');
            return true;
        } else {
            console.warn('⚠️ Cannot send message - WebSocket not connected');
            return false;
        }
    }, []);

    const disconnect = useCallback(() => {
        console.log('🧹 Manual disconnect');
        manualDisconnectRef.current = true;
        clearReconnectTimeout();
        if (wsRef.current) {
            wsRef.current.close(1000, 'Manual disconnect');
            wsRef.current = null;
        }
        setIsConnected(false);
        setConnectionState('disconnected');
    }, [clearReconnectTimeout]);

    const reconnect = useCallback(() => {
        console.log('🔄 Manual reconnect requested');
        manualDisconnectRef.current = false;
        setReconnectAttempts(0);
        clearReconnectTimeout();
        if (wsRef.current) {
            wsRef.current.close(1000, 'Reconnecting');
            wsRef.current = null;
        }
        connect();
    }, [connect, clearReconnectTimeout]);

    // Initial connection
    useEffect(() => {
        connect();

        return () => {
            console.log('🧹 Cleaning up WebSocket');
            clearReconnectTimeout();
            manualDisconnectRef.current = true;
            if (wsRef.current) {
                wsRef.current.close(1000, 'Component cleanup');
                wsRef.current = null;
            }
        };
    }, [url]); // Reconnect when URL changes

    return {
        isConnected,
        sendMessage,
        disconnect,
        reconnect,
        reconnectAttempts,
        connectionState,
    };
};
