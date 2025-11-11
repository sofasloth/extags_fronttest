<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
  import type { derived } from "svelte/store";
	import KeywordCollection from "../../Components/search_keyword/KeywordCollection.svelte"
	let {
		search_query_window_id,
		request_query,
	} = $props();
	interface Keyword{
		id: number,
		group_id : number,
		type: string,
		content: string,
		editable: boolean
	}
	interface KeywordStatistics{
		default: number,
		tag: number,
		relation: number,
		directory: number,
		extension: number,
		date: number
	}

	let search_content = $state('');
	let search_mode = $state('default');
	let keywords : Record<number, Keyword[]> = $state({}); // -1, 0 , 1,로 시작하고 기존 keywords는 그 키의 하위로..
	// let keywords_cnt : KeywordStatistics = $derived.by(() => {
	// 	let cnt = {default: 0, tag: 0, relation: 0, directory: 0, extension: 0, date: 0};
	// 	for(const key of Object.keys(keywords)) {
	// 		for(const word of keywords[Number(key)]) {
	// 			switch(word.type) {
	// 				case 'default' :
	// 					cnt.default += 1;
	// 					break;
	// 				case 'tag' : 
	// 					cnt.tag +=1;
	// 					break;
	// 				case 'relation' :
	// 					cnt.relation += 1;
	// 					break;
	// 				case 'directory' :
	// 					cnt.directory += 1;
	// 					break;
	// 				case 'extension' :
	// 					cnt.extension += 1;
	// 					break;
	// 				case 'date' :
	// 					cnt.date += 1;
	// 					break;
	// 			}
	// 		}
	// 	}
	// 	return cnt;
	// });
	let keywords_cnt : KeywordStatistics = $derived.by(() => {
		// 모든 키워드를 하나의 배열로 평탄화
		const allKeywords = Object.values(keywords).flat();	
		// reduce로 타입별 개수 계산
		return allKeywords.reduce((cnt, word) => {
			cnt[word.type as keyof KeywordStatistics] = (cnt[word.type as keyof KeywordStatistics] || 0) + 1;
			return cnt;
			}, {default: 0, tag: 0, relation: 0, directory: 0, extension: 0, date: 0} as KeywordStatistics);
	});
	// let groups = $state([]); // OR 연산자 CNF 패턴을 위해 묶어둘 예정
	let idx = $state(0);
	let group_ai = $state(1);
	let group_idx : number = $state(0);
	// keywords.set(-1,[]);
	keywords[0] = [];
	

	function determine_type(str : string) {
		let ret = 'default'
		if(str[0] === "#") {
			ret = 'tag'
		} else if(str[0] === "@") {
			ret = 'relation'
		} else if(str[0] === "/") {
			ret = 'directory'
		} else if(str[0] === ".") {
			ret = 'extension'
		} else if(str[0] === "~") {
			ret = 'date'
		} else {
			ret = 'default'
		}
		return ret
	}

	function triggerQuery() {
		// request_query(search_query_window_id, keywords);
		console.log("request search query")
		console.log("object size : ", $state.snapshot(keywords_cnt))
		// 구현방식은 rust에서 할 예정
		invoke('search_advanced', {searchId: 'mykey', keywords: keywords})
	}

	function search_keydown(){
		if (search_content.trim().length === 0 || (search_mode !== 'default' && search_content.trim().length === 1)) {
			search_mode = 'default'
			return;
		}
		search_mode = determine_type(search_content);
	}
	
	function execute_search(e) {
		if (e.key === 'Enter' && search_content.trim().length>0) {
      if(e.shiftKey) {
        // shift + enter 누를 시 OR 연산자 실행 (CNF)
        // group_idx === -1 ? keywords.size + 1 : group_idx
				if (group_idx === 0) {
					// group_idx = keywords.size
					group_idx = group_ai;
					// keywords.set(group_idx, [])
					// if (Object.values(keywords[group_idx]) === null)
					keywords[group_idx] = []
					group_ai++;
				}
      } else {
        // shift + enter를 안 누르면 OR 연산자 아님.
        group_idx = 0;
      }
			console.log(`enter : ${search_content}`)
			// keywords.set(group_idx, [...keywords.get(group_idx), {
			// 	id: idx,
      //   group_id : group_idx,
			// 	type: search_mode,
			// 	content: search_content,
			// 	editable: false,
			// }])
			keywords[group_idx] = [...(keywords[group_idx]), {
				id: idx,
        group_id : group_idx,
				type: search_mode,
				content: search_content,
				editable: false,
			}]

			idx += 1
			search_content = ''
			search_mode = 'default'
			console.log($state.snapshot(typeof(keywords[0])))
			console.log($state.snapshot(Object.keys(keywords)))
			optimize_search_keyword()
			
		}
	}

	function optimize_search_keyword() {
    let seen = new Map();
		let spread_keywords : Keyword[] = [];
		for(const values in Object.values(keywords)){
			spread_keywords = [...spread_keywords, ...values];
		}
		
    spread_keywords = spread_keywords.filter((keyword) => {
				// key를 두 개 설정하는 이유
				// 해당 검색창은 기본적으로 (A || B) && (C || D ) && !E 와 같은 CNF 방식의 복합 검색을 지원한다.
				// 여기서 (A || B) && (A|| C) 와 같은 검색은 A || (B && C) 와 같이 CNF 와 다른 결과를 반환하기 때문에 다른 그룹에 키를 추가하는 건 허용한다.
				// 반면 (A || B) && A 와 같은 경우, 사실상 A && B 와 다를바 없는 검색 결과가 나오기 때문에 group이 지정 안 되어 있는 경우는 무시한다.
        let no_group_key = `${keyword.type}:${keyword.content}:0`;
				let group_key = `${keyword.type}:${keyword.content}:${keyword.group_id}`;

        if(!seen.has(group_key)) {
            seen.set(group_key, true);
						seen.set(no_group_key, true);
            return true;
        }
        return false;
    });
		// 검색 요청.
		triggerQuery()
  }

	function remove_keyword(group_id: number, id: number) {
		keywords[group_id] = keywords[group_id].filter((ret) => 
			ret.id !== id
		)
		console.log(`remove id : ${id}`)
		if (keywords[group_id].length === 0 && group_id !== 0) {
			remove_group(group_id)
		}
    // console.log($state.snapshot(group_keywords))
		triggerQuery()
	}

	function remove_group(group_id: number) {
		// keywords = keywords.filter((id, value) => id !== group_id)
		// keywords = keywords[Object.keys(keywords).filter(key => Number(key) !== group_id)]
		const {[group_id] : _, ...rest} = keywords;
		keywords = rest;
		if(group_id === group_idx) {
			group_idx = 0;
		}
		triggerQuery()
	}
	
	function type_badge(type: string) {
		switch(type) {
			case 'tag':
				return '🎫'; //🎟️ ✨
			case 'relation':
				return '📎';
			case 'directory':
				return '📁';
			case 'extension':
				return '🧩';
			case 'date':
				return '🗓️'; //'📅';
			default:
				return '⚪';
		}
	}
</script>

<h1>Test Search vector</h1>
<div class="container">
	<div class="keywords-collection">
		<span class="collection-title">Search Statements</span>
		<div class="type-counter">
			{#each Object.keys(keywords_cnt) as key}
				<div class="type-counter-item">
					<span class="type-counter-item-badge">{type_badge(key)}</span>
					{keywords_cnt[key as keyof KeywordStatistics]}
				</div>
			{/each}
		</div>
		<div class="divider"></div>
		<input 
			type="text" 
			bind:value={search_content}
			class={`searchbar input-${search_mode}`}
			oninput={()=>{search_keydown()}}
			onkeydown={(e)=>{execute_search(e)}}
			>
		<div class="keyword-list">
			{#each Object.keys(keywords).filter(key => Number(key) !== 0) as key} 
			<!-- <div class="keyword-OR-group">
				{#each keywords[Number(key)] as keyword}
					<div class={`keyword keyword-${keyword.type}`}>
						<span class="type-badge">{keyword.type !== 'default' ? keyword.content[0] : ' '}</span>
						{#if !keyword.editable}
						<div class="keyword-content"
								ondblclick={()=>{keyword.editable = true}}>
							{keyword.type !== 'default' ? keyword.content.slice(1) : keyword.content}
						</div>
						{:else}
						<textarea 
								class="keyword-content-edit"
								onblur={()=>{keyword.editable = false}}
								oninput={()=>{
									keyword.type = determine_type(keyword.content)}}
								onkeydown={(e)=>{if(e.key === 'Enter'&&!e.shiftKey && !e.ctrlKey) {
										keyword.editable=false
									}
								}}
								onchange={()=> {
									keyword.content = keyword.content.replace(/\n+/g,'');
									if (keyword.content.trim().length ===0) remove_keyword(keyword.group_id, keyword.id)
								}}
								bind:value={keyword.content}
								autofocus></textarea>
						{/if}
						<span 
								class="close-btn"
								aria-label="remove button"
								role="button"
								onclick={()=>{remove_keyword(keyword.group_id, keyword.id)}}>×</span>
					</div>
				{/each}
			</div> -->
				<KeywordCollection 
					group_keywords={keywords[Number(key)]}
					is_group={true}
					group_idx={Number(key)}
					remove_keyword={(id: number) => remove_keyword(Number(key), id)}
					remove_group={() => remove_group(Number(key))}
					triggerQuery={()=>triggerQuery()}></KeywordCollection>
			{/each}
			<!-- {#each keywords[0] as keyword}
				<div class={`keyword keyword-${keyword.type}`}>
					<span class="type-badge">{keyword.type !== 'default' ? keyword.content[0] : ' '}</span>
					{#if !keyword.editable}
			    <div class="keyword-content"
							ondblclick={()=>{keyword.editable = true}}>
		        {keyword.type !== 'default' ? keyword.content.slice(1) : keyword.content}
					</div>
					{:else}
					<textarea 
							class="keyword-content-edit"
							onblur={()=>{keyword.editable = false}}
							oninput={()=>{
								keyword.type = determine_type(keyword.content)}}
							onkeydown={(e)=>{if(e.key === 'Enter'&&!e.shiftKey && !e.ctrlKey) {
									keyword.editable=false
								}
							}}
							onchange={()=> {
								keyword.content = keyword.content.replace(/\n+/g,'');
								if (keyword.content.trim().length ===0) remove_keyword(keyword.group_id, keyword.id)
							}}
							bind:value={keyword.content}
							autofocus></textarea>
					{/if}
					<span 
			        class="close-btn"
			        aria-label="remove button"
			        role="button"
			        onclick={()=>{remove_keyword(keyword.group_id, keyword.id)}}>×</span>
				</div>
			{/each} -->
			<KeywordCollection 
					group_keywords={keywords[0]}
					is_group={false}
					group_idx={0}
					remove_keyword={(id: number) => remove_keyword(0, id)}
					remove_group={() => remove_group(0)}
					triggerQuery={()=>triggerQuery()}></KeywordCollection>
		</div>
	</div>
	<div>
		
	</div>
</div>


<style>
	.container{
		display:flex;
		flex-direction:row;
		border: 1px dashed black;

		width: 90%;
    margin: 0 10px;
		min-height: 200px;
		max-height: 300px;
	}
	.keywords-collection {
    display: flex;
    flex-direction: column;
    justify-content: start;
    align-items: center;
    border: 1px dashed var(--border-color);
    border-radius: 8px;

    margin: 4px 8px;
    width: 200px; /* 고정 폭 설정 */
    max-width: 300px;
    height: inherit;
}
	.collection-title {
		font-family:'inter';
		font-weight:700;
	}
	.type-counter {
		box-sizing: border-box;
		display: flex;
		flex-direction: row;
		justify-content: start;
		align-items: center;
		width: 100%;
		height: 20px;
		/* background-color: var(--bg-secondary); */
		border-radius: 8px;
		padding: 4px 4px 4px 8px;
	}
	.type-counter-item {
		white-space: nowrap;
		
	}
	.searchbar{
		border: 1px dashed var(--border-color);
		border-radius: 8px;
		width: calc(90% - 12px);
		margin: 4px 0 4px 0;
		padding: 4px 4px 4px 8px;
    background-color: var(--bg-secondary);
	}
	.input-default {
		/* border: 1px solid white; */
		background-color: transparent;
	}
	.input-tag {
		background-color: #31522dcc;
		color: white;
	}
	.input-relation {
		background-color: #2d5244cc;
		color: white;
	}
	.input-directory {
		background-color: #2d4352cc;
		color: white;
	}
	.input-extension {
		background-color: #412d52cc;
		color: white;
	}
	.input-date {
		background-color: #522d41cc;
		color: white;
	}
	.divider {
		border:1px solid white;
		width: 90%;
		height: 0;
		margin: 3px 4px;
	}
	.keyword-list {
    display:flex;
    flex-direction: column;
    justify-content: start;
    align-items: center;
		width:100%;
		padding:0;
		flex: 1;
		overflow-x: hidden;
		overflow-y: scroll;
    transform: translateX(6px);
    font-size:0.85rem;
	}
	.keyword-list::-webkit-scrollbar {
    width: 4px;
    border-radius:2px;
		margin-left: 0;
	}
	::-webkit-scrollbar-thumb {
		background:#e89a3e;
	}
	.keyword {
    position: relative;
    display: block; /* flex 제거 */
    
    width: 80%;
    border: 1px solid white;
    border-radius: 8px;
		box-shadow: 2px 4px 4px #00000040;
    padding: 4px 4px 4px 1rem; /* 왼쪽 여백 확보 */
    margin: 3px 0 3px -8px;
    word-break: break-word;
		color:white;
	}
	.type-badge {
		user-select: none;
    position: absolute;
    left: 0.5rem;
    top: 0rem;
    font-weight: bold;
	}
	.keyword-content {
    display: inline; /* inline 텍스트 흐름 */
    
    /* 3줄 제한 */
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.1rem;
	}
	.keyword-content-edit{
		display: -webkit-box;
		border: none;
		outline:none;
		background-color: transparent;
		font-family: inherit;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.1rem;
		resize:none;
	}
	.close-btn {
    position: absolute;
    right: 4px;
    bottom: 4px; /* 항상 오른쪽 아래 고정 */
    
    width: 1rem;
    height: 1rem;
    /* border: 1px solid white; */
    border-radius: 8px;
    background-color: #2d2d2d;
    cursor: pointer;
    user-select: none;
    font-size: 1rem;
    line-height: 1.1rem;
    text-align: center;
		opacity: 50%;
		transition: background-color 0.2s ease, opacity 0.2s ease;
	}
	.close-btn:hover{
		background-color: #db394c;
		opacity:100%;
	}
	.close-btn:active {
		background-color: #7d1a26;
	}
	.keyword-default{
		background-color: #525252;
		/* color: black; */
	}
	.keyword-tag{
		background-color: #31522d;
	}
	.keyword-relation{
		background-color: #2d5244;
	}
	.keyword-directory {
		background-color: #2d4352;
	}
	.keyword-extension {
		background-color: #412d52;
	}
	.keyword-date {
		background-color: #522d41;
	}
	
</style>