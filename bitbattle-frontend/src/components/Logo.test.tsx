import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import Logo from './Logo';

describe('Logo', () => {
    it('renders without crashing', () => {
        render(<Logo />);
        // Logo contains an SVG element
        expect(document.querySelector('svg')).toBeInTheDocument();
    });

    it('renders without text by default', () => {
        render(<Logo />);
        expect(screen.queryByText('BitBattle')).not.toBeInTheDocument();
    });

    it('renders with text when showText is true', () => {
        render(<Logo showText={true} />);
        expect(screen.getByText('BitBattle')).toBeInTheDocument();
    });

    it('applies correct size for sm', () => {
        render(<Logo size="sm" />);
        const svg = document.querySelector('svg');
        expect(svg).toHaveAttribute('width', '24');
        expect(svg).toHaveAttribute('height', '24');
    });

    it('applies correct size for md', () => {
        render(<Logo size="md" />);
        const svg = document.querySelector('svg');
        expect(svg).toHaveAttribute('width', '28');
        expect(svg).toHaveAttribute('height', '28');
    });

    it('applies correct size for lg', () => {
        render(<Logo size="lg" />);
        const svg = document.querySelector('svg');
        expect(svg).toHaveAttribute('width', '36');
        expect(svg).toHaveAttribute('height', '36');
    });

    it('applies correct size for xl', () => {
        render(<Logo size="xl" />);
        const svg = document.querySelector('svg');
        expect(svg).toHaveAttribute('width', '80');
        expect(svg).toHaveAttribute('height', '80');
    });

    it('defaults to md size', () => {
        render(<Logo />);
        const svg = document.querySelector('svg');
        expect(svg).toHaveAttribute('width', '28');
        expect(svg).toHaveAttribute('height', '28');
    });

    it('renders SVG with correct viewBox', () => {
        render(<Logo />);
        const svg = document.querySelector('svg');
        expect(svg).toHaveAttribute('viewBox', '0 0 32 32');
    });

    it('renders the butterfly/BB logo shape', () => {
        render(<Logo />);
        const paths = document.querySelectorAll('path');
        // Logo has 6 path elements (4 wings + 2 antennae)
        expect(paths.length).toBe(6);
    });
});
