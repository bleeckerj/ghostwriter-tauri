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

    flash() {
        this.element.classList.add('flash');
        setTimeout(() => {
            this.element.classList.remove('flash');
        }, 1000);
    }

    start(onTick = null, onComplete = null) {
        this.onTick = onTick;
        this.onComplete = onComplete;
        this.element.classList.remove('hidden');
        
        if (this.intervalId) return;

        this.intervalId = setInterval(() => {
            if (this.currentTime > 0) {
                this.currentTime--;
                this.updateDisplay();
                this.flash();
                if (this.onTick) this.onTick(this.currentTime);
            } else {
                this.stop();
                if (this.onComplete) this.onComplete();
            }
        }, 1000);
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
}