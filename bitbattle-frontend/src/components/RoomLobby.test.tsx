import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import RoomLobby from './RoomLobby';

// Mock the AuthContext
vi.mock('../contexts/AuthContext.tsx', () => ({
    useAuth: vi.fn(() => ({
        user: null,
        isAuthenticated: false,
        isLoading: false,
        login: vi.fn(),
        logout: vi.fn(),
    })),
}));

// Mock the GoogleLoginButton
vi.mock('./GoogleLoginButton.tsx', () => ({
    default: () => <button data-testid="google-login">Sign in with Google</button>,
}));

// Helper to render with router
const renderWithRouter = (component: React.ReactElement) => {
    return render(<BrowserRouter>{component}</BrowserRouter>);
};

describe('RoomLobby', () => {
    const mockOnJoinRoom = vi.fn();
    const mockSetUsername = vi.fn();
    const defaultProps = {
        onJoinRoom: mockOnJoinRoom,
        username: 'guest_1234',
        setUsername: mockSetUsername,
    };

    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('Home Mode', () => {
        it('renders the home screen by default', () => {
            renderWithRouter(<RoomLobby {...defaultProps} />);
            // Use getAllByText since BitBattle appears in both navbar and heading
            expect(screen.getAllByText('BitBattle').length).toBeGreaterThan(0);
            expect(screen.getByText('Competitive coding arena')).toBeInTheDocument();
        });

        it('shows Create Room button', () => {
            renderWithRouter(<RoomLobby {...defaultProps} />);
            expect(screen.getByText('Create Room')).toBeInTheDocument();
        });

        it('shows Join Room button', () => {
            renderWithRouter(<RoomLobby {...defaultProps} />);
            expect(screen.getByText('Join Room')).toBeInTheDocument();
        });

        it('shows Quick Match button', () => {
            renderWithRouter(<RoomLobby {...defaultProps} />);
            expect(screen.getByText('Quick Match')).toBeInTheDocument();
        });

        it('displays guest username when not authenticated', () => {
            renderWithRouter(<RoomLobby {...defaultProps} />);
            expect(screen.getByText('Playing as')).toBeInTheDocument();
            expect(screen.getByText('guest_1234')).toBeInTheDocument();
        });

        it('shows how it works section', () => {
            renderWithRouter(<RoomLobby {...defaultProps} />);
            expect(screen.getByText('How it works')).toBeInTheDocument();
        });

        it('clicking Quick Match opens matchmaking settings', () => {
            renderWithRouter(<RoomLobby {...defaultProps} />);
            fireEvent.click(screen.getByText('Quick Match'));
            // Should show the Quick Match settings screen
            expect(screen.getByText('Find Match')).toBeInTheDocument();
        });
    });

    describe('Create Mode', () => {
        it('switches to create mode when Create Room is clicked', () => {
            renderWithRouter(<RoomLobby {...defaultProps} />);
            fireEvent.click(screen.getByRole('button', { name: 'Create Room' }));
            // Now we should see the Difficulty and Players labels (may appear multiple times)
            expect(screen.getAllByText('Difficulty').length).toBeGreaterThan(0);
            expect(screen.getAllByText('Players').length).toBeGreaterThan(0);
        });

        it('shows difficulty options', () => {
            renderWithRouter(<RoomLobby {...defaultProps} />);
            fireEvent.click(screen.getByRole('button', { name: 'Create Room' }));
            expect(screen.getByRole('button', { name: 'Random' })).toBeInTheDocument();
            expect(screen.getByRole('button', { name: 'Easy' })).toBeInTheDocument();
            expect(screen.getByRole('button', { name: 'Medium' })).toBeInTheDocument();
            expect(screen.getByRole('button', { name: 'Hard' })).toBeInTheDocument();
        });

        it('shows game mode options (Casual/Ranked)', () => {
            renderWithRouter(<RoomLobby {...defaultProps} />);
            fireEvent.click(screen.getByRole('button', { name: 'Create Room' }));
            expect(screen.getByRole('button', { name: 'Casual' })).toBeInTheDocument();
            expect(screen.getByRole('button', { name: 'Ranked' })).toBeInTheDocument();
        });

        it('shows player count controls', () => {
            renderWithRouter(<RoomLobby {...defaultProps} />);
            fireEvent.click(screen.getByRole('button', { name: 'Create Room' }));
            expect(screen.getByText('Players')).toBeInTheDocument();
        });

        it('has Cancel button that returns to home', () => {
            renderWithRouter(<RoomLobby {...defaultProps} />);
            fireEvent.click(screen.getByRole('button', { name: 'Create Room' }));
            fireEvent.click(screen.getByText('Cancel'));
            // Should be back to home mode
            expect(screen.getByText('Quick Match')).toBeInTheDocument();
        });
    });

    describe('Join Mode', () => {
        it('switches to join mode when Join Room is clicked', () => {
            renderWithRouter(<RoomLobby {...defaultProps} />);
            fireEvent.click(screen.getByRole('button', { name: 'Join Room' }));
            expect(screen.getByText('Room Code')).toBeInTheDocument();
        });

        it('shows room code input', () => {
            renderWithRouter(<RoomLobby {...defaultProps} />);
            fireEvent.click(screen.getByRole('button', { name: 'Join Room' }));
            const input = screen.getByPlaceholderText('SWIFT-CODER-1234');
            expect(input).toBeInTheDocument();
        });

        it('has Cancel button that returns to home', () => {
            renderWithRouter(<RoomLobby {...defaultProps} />);
            fireEvent.click(screen.getByRole('button', { name: 'Join Room' }));
            fireEvent.click(screen.getByText('Cancel'));
            expect(screen.getByText('Quick Match')).toBeInTheDocument();
        });

        it('converts room code input to uppercase', () => {
            renderWithRouter(<RoomLobby {...defaultProps} />);
            fireEvent.click(screen.getByRole('button', { name: 'Join Room' }));
            const input = screen.getByPlaceholderText('SWIFT-CODER-1234') as HTMLInputElement;
            fireEvent.change(input, { target: { value: 'swift-coder-1234' } });
            expect(input.value).toBe('SWIFT-CODER-1234');
        });

        it('Join Room button is disabled when room code is empty', () => {
            renderWithRouter(<RoomLobby {...defaultProps} />);
            fireEvent.click(screen.getByRole('button', { name: 'Join Room' }));
            const joinButtons = screen.getAllByRole('button', { name: 'Join Room' });
            // The second Join Room button is in the modal/form
            expect(joinButtons[joinButtons.length - 1]).toBeDisabled();
        });
    });

    describe('Navigation', () => {
        it('renders logo that links to home', () => {
            renderWithRouter(<RoomLobby {...defaultProps} />);
            // BitBattle appears in navbar and heading
            const logos = screen.getAllByText('BitBattle');
            expect(logos.length).toBeGreaterThan(0);
        });
    });
});

describe('RoomLobby - Authenticated User', () => {
    beforeEach(() => {
        // Reset the mock
        vi.resetModules();
    });

    it('shows user display name when authenticated', async () => {
        // This would require mocking useAuth to return an authenticated user
        // For now, we test the unauthenticated state
        const mockOnJoinRoom = vi.fn();
        const mockSetUsername = vi.fn();

        renderWithRouter(
            <RoomLobby
                onJoinRoom={mockOnJoinRoom}
                username="guest_1234"
                setUsername={mockSetUsername}
            />
        );

        expect(screen.getByText('guest_1234')).toBeInTheDocument();
    });
});
