import { Controller, Get, Param, Query } from '@nestjs/common';
import { ApiOperation, ApiQuery } from '@nestjs/swagger';
import { TokensService } from './tokens.service';
import { Token } from './entities/token.entity';

@Controller('tokens')
export class TokensController {
  constructor(private readonly tokensService: TokensService) {}

  @Get()
  @ApiOperation({
    summary: 'Returns the active token list for frontend dropdowns',
  })
  @ApiQuery({
    name: 'whitelisted',
    required: false,
    type: Boolean,
    description: 'If true, only returns whitelisted tokens',
  })
  async getTokens(
    @Query('whitelisted') whitelisted?: string,
  ): Promise<Token[]> {
    const whitelistedOnly = whitelisted === 'true';
    return this.tokensService.getAll(whitelistedOnly);
  }

  @Get('search')
  async searchTokens(@Query('q') query: string) {
    return this.tokensService.searchDexPairs(query ?? '');
  }

  @Get(':pair/price')
  async getPairPrice(@Param('pair') pair: string) {
    return this.tokensService.getPairPrice(pair);
  }
}
