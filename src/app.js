const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { open } = window.__TAURI__.dialog;
const { readTextFile } = window.__TAURI__.fs;

class WordRemberApp {
    constructor() {
        this.words = [];
        this.allWords = [];
        this.learnedWords = [];
        this.filteredWords = [];
        this.currentPage = 1;
        this.pageSize = 20;
        this.totalPages = 1;
        this.currentTab = 'all';
        this.selectedFileContent = null;
        this.settings = {
            memory_threshold: 5
        };
        this.init();
    }

    async init() {
        await this.loadSettings();
        await this.loadWords();
        this.bindEvents();
        this.updateUI();
    }

    async loadSettings() {
        try {
            const settings = await invoke('get_settings');
            this.settings = settings;
            this.applySettings();
        } catch (e) {
            console.error('Failed to load settings:', e);
        }
    }

    applySettings() {
        document.getElementById('settingThreshold').value = this.settings.memory_threshold;
    }

    async loadWords() {
        try {
            this.words = await invoke('get_words');
            this.allWords = await invoke('get_all_words');
            this.learnedWords = await invoke('get_learned_words');
            this.filterByTab();
            this.renderWords();
            this.updateUI();
        } catch (e) {
            console.error('Failed to load words:', e);
        }
    }

    filterByTab() {
        switch (this.currentTab) {
            case 'all':
                this.filteredWords = this.allWords;
                break;
            case 'pending':
                this.filteredWords = this.words;
                break;
            case 'learned':
                this.filteredWords = this.learnedWords;
                break;
            default:
                this.filteredWords = this.allWords;
        }
        this.totalPages = Math.ceil(this.filteredWords.length / this.pageSize) || 1;
        if (this.currentPage > this.totalPages) {
            this.currentPage = this.totalPages;
        }
    }

    switchPage(page) {
        this.currentTab = page;
        this.currentPage = 1;
        
        document.querySelectorAll('.nav-tab').forEach(btn => {
            btn.classList.toggle('active', btn.dataset.page === page);
        });
        
        const wordGrid = document.getElementById('wordGrid');
        const pagination = document.getElementById('pagination');
        const pageManage = document.getElementById('pageManage');
        const pageSettings = document.getElementById('pageSettings');
        
        wordGrid.style.display = 'grid';
        pagination.style.display = 'flex';
        pageManage.style.display = 'none';
        pageSettings.style.display = 'none';
        
        if (page === 'manage') {
            wordGrid.style.display = 'none';
            pagination.style.display = 'none';
            pageManage.style.display = 'block';
            this.updateHeaderCount('单词管理');
        } else if (page === 'settings') {
            wordGrid.style.display = 'none';
            pagination.style.display = 'none';
            pageSettings.style.display = 'block';
            this.updateHeaderCount('系统设置');
        } else {
            this.filterByTab();
            this.renderWords();
            this.updateUI();
        }
    }

    switchManageTab(tab) {
        document.querySelectorAll('.manage-tab').forEach(btn => {
            btn.classList.toggle('active', btn.dataset.manage === tab);
        });
        document.getElementById('manageAdd').style.display = tab === 'add' ? 'block' : 'none';
        document.getElementById('manageImport').style.display = tab === 'import' ? 'block' : 'none';
    }

    updateHeaderCount(text) {
        document.getElementById('headerCount').textContent = text;
    }

    getCurrentPageWords() {
        const start = (this.currentPage - 1) * this.pageSize;
        const end = start + this.pageSize;
        return this.filteredWords.slice(start, end);
    }

    renderWords() {
        const grid = document.getElementById('wordGrid');
        const pageWords = this.getCurrentPageWords();

        if (pageWords.length === 0) {
            let emptyMessage = '';
            let emptyIcon = '📚';
            switch (this.currentTab) {
                case 'all':
                    emptyMessage = '暂无单词，点击"单词管理"添加';
                    emptyIcon = '📚';
                    break;
                case 'pending':
                    emptyMessage = '太棒了！所有单词都已掌握！';
                    emptyIcon = '🎉';
                    break;
                case 'learned':
                    emptyMessage = '还没有已掌握的单词，继续努力学习吧！';
                    emptyIcon = '💪';
                    break;
                default:
                    emptyMessage = '暂无单词';
                    emptyIcon = '📚';
            }
            grid.innerHTML = `
                <div class="empty-state">
                    <div class="icon">${emptyIcon}</div>
                    <h3>${emptyMessage}</h3>
                </div>
            `;
            return;
        }

        grid.innerHTML = pageWords.map((word, index) => {
            const level = Math.min(word.success_count, 5);
            const progress = Math.round((word.success_count / word.threshold) * 100);
            return `
                <div class="word-card" data-id="${word.id}" data-level="${level}" style="animation-delay: ${index * 0.05}s">
                    <div class="word-text">${this.escapeHtml(word.word)}</div>
                    <div class="success-count">
                        🔥 <span class="count-badge">${word.success_count}</span>
                        <span style="color: var(--success-color)">(${progress}%)</span>
                    </div>
                    <div class="meaning-hidden" data-word-id="${word.id}">点击查看意思</div>
                    <div class="card-actions">
                        <button class="btn-remember remembered" data-action="remember" data-id="${word.id}">
                            ✓ 记住了
                        </button>
                        <button class="btn-remember forgot" data-action="forgot" data-id="${word.id}">
                            ✗ 忘了
                        </button>
                        <button class="btn-delete" data-action="delete" data-id="${word.id}">🗑️</button>
                    </div>
                </div>
            `;
        }).join('');
    }

    updateUI() {
        if (['all', 'pending', 'learned'].includes(this.currentTab)) {
            const count = this.filteredWords.length;
            let label = '';
            switch (this.currentTab) {
                case 'all': label = '全部单词'; break;
                case 'pending': label = '待掌握'; break;
                case 'learned': label = '已掌握'; break;
            }
            this.updateHeaderCount(`${label}: ${count} 个`);
        }
        
        document.getElementById('pageInfo').textContent = `${this.currentPage} / ${this.totalPages}`;
        document.getElementById('btnPrev').disabled = this.currentPage <= 1;
        document.getElementById('btnNext').disabled = this.currentPage >= this.totalPages;
    }

    bindEvents() {
        document.getElementById('navAll').addEventListener('click', () => this.switchPage('all'));
        document.getElementById('navPending').addEventListener('click', () => this.switchPage('pending'));
        document.getElementById('navLearned').addEventListener('click', () => this.switchPage('learned'));
        document.getElementById('navManage').addEventListener('click', () => this.switchPage('manage'));
        document.getElementById('navSettings').addEventListener('click', () => this.switchPage('settings'));

        document.getElementById('manageTabAdd').addEventListener('click', () => this.switchManageTab('add'));
        document.getElementById('manageTabImport').addEventListener('click', () => this.switchManageTab('import'));

        document.getElementById('btnPrev').addEventListener('click', () => this.prevPage());
        document.getElementById('btnNext').addEventListener('click', () => this.nextPage());

        document.getElementById('btnSubmitWord').addEventListener('click', () => this.addWord());
        document.getElementById('btnDoImport').addEventListener('click', () => this.importWords());
        document.getElementById('btnSelectFile').addEventListener('click', () => this.selectImportFile());
        document.getElementById('btnResetProgress').addEventListener('click', () => this.resetProgress());
        document.getElementById('btnCloseMeaning').addEventListener('click', () => this.hideModal('modalMeaning'));

        document.getElementById('pageSize').addEventListener('change', (e) => {
            this.pageSize = parseInt(e.target.value);
            this.currentPage = 1;
            this.renderWords();
            this.updateUI();
        });

        document.getElementById('wordGrid').addEventListener('click', (e) => {
            const action = e.target.dataset.action;
            const id = parseInt(e.target.dataset.id);

            if (action === 'remember') {
                this.markRemembered(id);
            } else if (action === 'forgot') {
                this.markNotRemembered(id);
            } else if (action === 'delete') {
                this.deleteWord(id);
            } else if (e.target.classList.contains('meaning-hidden')) {
                this.revealMeaning(e.target);
            }
        });

        document.addEventListener('keydown', (e) => {
            if (e.key === 'ArrowLeft') {
                this.prevPage();
            } else if (e.key === 'ArrowRight') {
                this.nextPage();
            } else if (e.key === 'Escape') {
                this.closeAllModals();
            }
        });
    }

    prevPage() {
        if (this.currentPage > 1) {
            this.currentPage--;
            this.renderWords();
            this.updateUI();
        }
    }

    nextPage() {
        if (this.currentPage < this.totalPages) {
            this.currentPage++;
            this.renderWords();
            this.updateUI();
        }
    }

    async markRemembered(id) {
        try {
            await invoke('mark_remembered', { wordId: id });
            await this.loadWords();
        } catch (e) {
            console.error('Failed to mark remembered:', e);
        }
    }

    async markNotRemembered(id) {
        try {
            await invoke('mark_not_remembered', { wordId: id });
            await this.loadWords();
        } catch (e) {
            console.error('Failed to mark not remembered:', e);
        }
    }

    async deleteWord(id) {
        console.log('Attempting to delete word with id:', id);
        if (confirm('确定要删除这个单词吗？')) {
            try {
                console.log('Calling delete_word command...');
                await invoke('delete_word', { wordId: id });
                console.log('Word deleted successfully, reloading words...');
                await this.loadWords();
                console.log('Words reloaded');
            } catch (e) {
                console.error('Failed to delete word:', e);
                alert('删除失败：' + e);
            }
        }
    }

    revealMeaning(element) {
        const wordId = parseInt(element.dataset.wordId);
        const word = this.allWords.find(w => w.id === wordId) || 
                     this.words.find(w => w.id === wordId) || 
                     this.learnedWords.find(w => w.id === wordId);
        if (word) {
            element.textContent = word.meaning;
            element.classList.add('revealed');
        }
    }

    async addWord() {
        const wordInput = document.getElementById('inputWord');
        const meaningInput = document.getElementById('inputMeaning');
        const word = wordInput.value.trim();
        const meaning = meaningInput.value.trim();

        if (!word || !meaning) {
            alert('请输入单词和意思');
            return;
        }

        try {
            await invoke('add_word', { word, meaning });
            wordInput.value = '';
            meaningInput.value = '';
            alert('添加成功！');
            await this.loadWords();
        } catch (e) {
            console.error('Failed to add word:', e);
            alert('添加失败: ' + e);
        }
    }

    async selectImportFile() {
        try {
            const selected = await open({
                multiple: false,
                filters: [{
                    name: '文本文件',
                    extensions: ['csv', 'json', 'txt']
                }]
            });

            if (selected) {
                const filePath = selected;
                const fileName = filePath.split('\\').pop().split('/').pop();
                document.getElementById('selectedFileName').textContent = fileName;

                const ext = fileName.split('.').pop().toLowerCase();
                const formatSelect = document.getElementById('selectFormat');
                if (ext === 'csv') formatSelect.value = 'csv';
                else if (ext === 'json') formatSelect.value = 'json';
                else if (ext === 'txt') formatSelect.value = 'txt';

                const content = await readTextFile(filePath);
                this.selectedFileContent = content;
                document.getElementById('inputImportContent').value = content;
            }
        } catch (e) {
            console.error('Failed to select file:', e);
            alert('选择文件失败: ' + e);
        }
    }

    async importWords() {
        const format = document.getElementById('selectFormat').value;
        const content = document.getElementById('inputImportContent').value.trim();

        if (!content) {
            alert('请输入要导入的内容或选择文件');
            return;
        }

        try {
            const count = await invoke('import_words', { content, format });
            alert(`成功导入 ${count} 个单词`);
            document.getElementById('inputImportContent').value = '';
            document.getElementById('selectedFileName').textContent = '未选择文件';
            this.selectedFileContent = null;
            await this.loadWords();
        } catch (e) {
            console.error('Failed to import words:', e);
            alert('导入失败: ' + e);
        }
    }

    async resetProgress() {
        if (confirm('确定要重置所有学习进度吗？此操作不可恢复。')) {
            try {
                await invoke('reset_progress');
                alert('学习进度已重置');
                await this.loadWords();
            } catch (e) {
                console.error('Failed to reset progress:', e);
                alert('重置失败: ' + e);
            }
        }
    }

    hideModal(modalId) {
        document.getElementById(modalId).classList.remove('active');
    }

    closeAllModals() {
        document.querySelectorAll('.modal').forEach(modal => modal.classList.remove('active'));
    }

    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
}

const app = new WordRemberApp();
