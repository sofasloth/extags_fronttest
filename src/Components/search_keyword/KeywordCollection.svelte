<script lang="ts">
  import {fade, slide} from "svelte/transition";
  let {
    group_keywords = $bindable(),  // 전달받는 그룹
    is_group, 
    group_idx = 0,
    remove_keyword, // function
    remove_group, // function
    triggerQuery,
  } = $props();
  let default_color : string = '#335A96';
  let is_fold = $state(false);
  let is_hover = $state(false);

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

<!-- 0이면 그룹 자체를 삭제할 것... -->
<div class="keyword-group-wrapper"
  onmouseover={()=>is_hover = true}
  onmouseleave={()=>is_hover=false}
  in:fade={{duration:200}}
  out:fade={{duration:200}}>
{#if is_group}
  <div class="keyword keyword-groupheader" class:group-hover={is_hover} onclick={()=>{is_fold=!is_fold}}>
    Group {group_idx}
    <span class="notification-banner">
      {group_keywords.length }
    </span>
    <span 
      class="close-btn"
      aria-label="remove button"
      role="button"
      onclick={()=>{remove_group()}}>×</span>
  </div>
{/if}
{#if !is_fold}
<div 
  class="keyword-wrapper"
  class:OR-group={is_group}
  in:slide={{duration : 200}}
  out:slide={{duration : 200}}>
  {#if is_group}
    <div class="group-guideline" class:group-hover={is_hover}></div>
  {/if}
    {#each group_keywords as keyword}
      <div class={`keyword keyword-${keyword.type}`}
        class:grouped-keyword={is_group}
        in:fade={{duration:200}}
        out:fade={{duration:200}}>
        <span class="type-badge">{type_badge(keyword.type)}</span>
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
              triggerQuery()
              if (keyword.content.trim().length ===0) remove_keyword(keyword.id)
            }}
            bind:value={keyword.content}
            autofocus></textarea>
        {/if}
        <span 
            class="close-btn"
            aria-label="remove button"
            role="button"
            onclick={()=>{remove_keyword(keyword.id)}}>×</span>
      </div>
    {/each}
</div>
{/if}
</div>


<style>
  .keyword-group-wrapper{
    box-sizing: border-box;
    width: 100%;
    height: fit-content;
    margin:3px 0 4px 0;
    padding: 0;
  }
  .OR-group {
    position: relative;
    display:flex;
    flex-direction: column;
    justify-content: start;
    align-items: center;
    width: 100%;
    height: fit-content;
    padding: 0 0 0 4px;
    margin: 4px 0;

    border: none;
    background-color: transparent;
  }
  .keyword-wrapper {
    position: relative;
    display:flex;
    flex-direction: column;
    justify-content: start;
    align-items: center;
    width : 100%;
    height: fit-content;
    margin : 0;
    padding: 0;
    /* margin : 4px 0; */
    
    border: none;
    background-color: transparent;
  }
  .group-guideline{
    position: absolute;
    top: 0px;
    left : 0px;
    width: 4px;
    height: calc(100%);
    background-color: #335A96;
    box-shadow: 2px 4px 4px #00000040;
    opacity: 0.8;
    transition: opacity 0.2s ease;
  }
  .group-hover{
    opacity: 1.0 !important;
  }
  .keyword-groupheader {
    width: 95% !important;
    margin:0 0 0 0 !important;
    border: none;
    border-radius: 0 12px 12px 0 !important;
    background-color: #335A96;
    user-select: none;
    cursor: pointer;
    font-weight: 700;
  }
  .keyword {
    position: relative;
    /* display: block; flex 제거 */
    display: flex;
    flex-direction: row;
    align-items: start;
    justify-content: start;
    
    box-sizing: border-box;
    width: 90%;
    /* border: 1px solid white; */
    border-radius: 12px;
		box-shadow: 2px 4px 4px #00000040;
    padding: 4px 4px 4px 4px; /* 왼쪽 여백 확보 */
    margin: 3px 0 3px 0px;
    word-break: break-word;
		color:white;
    opacity: 0.8;
    transition: opacity 0.2s ease;
	}
  .keyword:hover{
    opacity: 1.0;
  }
  .notification-banner{
    display: inline-block;
    padding: 0 4px;
    /* width:fit-content; */
    width: 1.1rem;
    /* height: 1.1rem; */
    border-radius: 100px;
    background-color: #f0113d;
    text-align: center;
    vertical-align: baseline;
    line-height: normal;
  }
  .grouped-keyword {
    border-radius: 4px 12px 12px 4px !important;
  }
	.type-badge {
    display: inline-block;
		user-select: none;
    /* left: 4px;
    top: 2px; */
    margin: -2px 0 0 0;
    /* vertical-align: text-top */
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
    user-select: none;
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