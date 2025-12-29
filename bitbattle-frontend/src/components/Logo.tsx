interface LogoProps {
    size?: 'sm' | 'md' | 'lg' | 'xl';
    showText?: boolean;
}

export default function Logo({ size = 'md', showText = false }: LogoProps) {
    const sizes = {
        sm: { icon: 24, title: 'text-base' },
        md: { icon: 28, title: 'text-lg' },
        lg: { icon: 36, title: 'text-xl' },
        xl: { icon: 80, title: 'text-3xl' },
    };

    const s = sizes[size];

    return (
        <div className="flex items-center gap-2">
            <svg
                width={s.icon}
                height={s.icon}
                viewBox="0 0 32 32"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
            >
                {/* Left wing (mirrored B) - top */}
                <path
                    d="M16 16 C16 10 14 6 10 4 C5 2 2 6 2 10 C2 14 6 16 16 16"
                    stroke="#d4d4d8"
                    strokeWidth="1.2"
                    fill="none"
                />
                {/* Left wing (mirrored B) - bottom */}
                <path
                    d="M16 16 C16 22 14 26 10 28 C5 30 1 26 1 21 C1 16 6 16 16 16"
                    stroke="#d4d4d8"
                    strokeWidth="1.2"
                    fill="none"
                />
                {/* Right wing (normal B) - top */}
                <path
                    d="M16 16 C16 10 18 6 22 4 C27 2 30 6 30 10 C30 14 26 16 16 16"
                    stroke="#d4d4d8"
                    strokeWidth="1.2"
                    fill="none"
                />
                {/* Right wing (normal B) - bottom */}
                <path
                    d="M16 16 C16 22 18 26 22 28 C27 30 31 26 31 21 C31 16 26 16 16 16"
                    stroke="#d4d4d8"
                    strokeWidth="1.2"
                    fill="none"
                />
                {/* Body/spine */}
                <line x1="16" y1="4" x2="16" y2="28" stroke="#d4d4d8" strokeWidth="1.2" strokeLinecap="round" />
                {/* Antennae */}
                <path d="M16 4 C14 2 12 1 10 2" stroke="#d4d4d8" strokeWidth="1" strokeLinecap="round" fill="none" />
                <path d="M16 4 C18 2 20 1 22 2" stroke="#d4d4d8" strokeWidth="1" strokeLinecap="round" fill="none" />
            </svg>
            {showText && (
                <span className={`${s.title} font-bold text-white`}>BitBattle</span>
            )}
        </div>
    );
}
