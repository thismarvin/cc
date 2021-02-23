augroup local
	autocmd!
	autocmd Filetype rust set makeprg=make\ $*
	nnoremap <F5> :Dispatch make dev<CR>
augroup end
