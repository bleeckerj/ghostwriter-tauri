export class Timer {
    constructor() {
        this.element = document.getElementById('floating-timer');
        this.intervalId = null;
        this.currentTime = 0;
        this.onTick = null;
        this.onComplete = null;
    }

    setTime(seconds) {
        this.currentTime = seconds;
        this.updateDisplay();
    }

    formatTime(seconds) {
        const minutes = Math.floor(seconds / 60);
        const remainingSeconds = seconds % 60;
        return `${minutes.toString().padStart(2, '0')}:${remainingSeconds.toString().padStart(2, '0')}`;
    }

    updateDisplay() {
        this.element.textContent = this.formatTime(this.currentTime);
    }

    flash(duration = 750) {
        this.element.classList.add('flash');
        setTimeout(() => {
            this.element.classList.remove('flash');
        }, duration);
    }

    start(onTick = null, onComplete = null) {
        this.onTick = onTick;
        this.onComplete = onComplete;
        
        if (this.intervalId) return;

        this.intervalId = setInterval(() => {
            if (this.currentTime > 0) {
                this.currentTime--;
                this.updateDisplay();
                if (this.currentTime <= 3) {
                    this.flash(750); // Flash for 750ms during the last 3 seconds
                }
                if (this.onTick) this.onTick(this.currentTime);
            } else {
                this.stop();
                if (this.onComplete) this.onComplete();
            }
        }, 1000);
        
        this.element.classList.remove('hidden');

    }

    stop() {
        if (this.intervalId) {
            clearInterval(this.intervalId);
            this.intervalId = null;
        }
    }

    hide() {
        this.element.classList.add('hidden');
    }

    show() {
        this.element.classList.remove('hidden');
    }

    isRunning() {
        return this.intervalId !== null;
    }
}